
use serde::{de::DeserializeOwned, Serialize};
use serde_derive::{Deserialize, Serialize};
use simple_wal::LogError;
use std::hash::Hash;
use tokio::{sync::mpsc::Sender, task::JoinHandle};

use dashmap::{iter::Iter, mapref::one::Ref, DashMap, DashSet};


use super::{
    disk_log::{DiskLog, Session},
    index::{hash::HashIndex, range::RangeIndex, tags::TagIndex, inverted_index::InvertedIndex},
    router::{self, Router},
    Options, StatusResult, StorageType,
};

use crate::{darkbird::SessionResult, document::{Document, GetContent}};

pub struct Storage<K, Doc: Document> {
    // DashMap
    collection: DashMap<K, Doc>,

    // HashIndex
    hash_index: HashIndex<K>,

    // TagIndex
    tag_index: TagIndex<K>,

    // RangeIndex
    range_index: RangeIndex<K>,

    // InvertedIndex
    inverted_index: InvertedIndex<K>,

    // Wal session
    wal_session: Option<Session>,

    // Reporter session
    reporter_session: router::Session<Event<K, Doc>>,

    off_reporter: bool,
}

impl<K, Doc> Storage<K, Doc>
where
    Doc: Serialize + DeserializeOwned + Clone + Send + 'static + Document,
    K: Serialize
        + DeserializeOwned
        + PartialOrd
        + Ord
        + PartialEq
        + Eq
        + Hash
        + Clone
        + Send
        + Sync
        + 'static,
{
    pub async fn open<'a>(ops: Options<'a>) -> Result<Self, LogError> {
        if let StorageType::DiskCopies = ops.stype {
            match DiskLog::open(ops.path, ops.storage_name, ops.total_page_size) {
                Err(e) => return Err(e),
                Ok(disklog) => {
                    // Run DiskLog
                    let wal_session = disklog.run_service();

                    // Run Reporter
                    let reporter = Router::<Event<K, Doc>>::new(vec![]).unwrap().run_service();

                    // Create Storage
                    let st = Storage {
                        collection: DashMap::new(),
                        hash_index: HashIndex::new(),
                        tag_index: TagIndex::new(),
                        range_index: RangeIndex::new(),
                        inverted_index: InvertedIndex::new(),
                        wal_session: Some(wal_session),
                        reporter_session: reporter,
                        off_reporter: false,
                    };

                    // load from disk
                    st.loader().await;

                    return Ok(st);
                }
            }
        } else {
            // Off DiskLog

            // Run Reporter
            let reporter = Router::<Event<K, Doc>>::new(vec![]).unwrap().run_service();

            // Create Storage
            let st = Storage {
                collection: DashMap::new(),
                hash_index: HashIndex::new(),
                tag_index: TagIndex::new(),
                range_index: RangeIndex::new(),
                inverted_index: InvertedIndex::new(),
                wal_session: None,
                reporter_session: reporter,
                off_reporter: false,
            };

            // loader off

            return Ok(st);
        }
    }

    #[inline]
    pub fn off_reporter(&mut self) {
        self.off_reporter = true;
    }

    /// subscribe to Reporter
    #[inline]
    pub async fn subscribe(&self, sender: Sender<Event<K, Doc>>) -> Result<(), SessionResult> {
        if self.off_reporter {
            return Err(SessionResult::Err(StatusResult::ReporterIsOff));
        }

        // Send to Reporter
        let _ = self
            .reporter_session
            .dispatch(Event::Subscribed(sender.clone()))
            .await;

        self.reporter_session.register(sender).await
    }

    /// insert to storage and persist to disk
    #[inline]
    pub async fn insert(&self, key: K, doc: Doc) -> Result<(), SessionResult> {

        if let Some(wal) = &self.wal_session {
            let query = RQuery::Insert(key.clone(), doc.clone());
            
            if let Err(e) = wal.log(bincode::serialize(&query).unwrap()).await {
                return Err(e);
            }
            
            if !self.off_reporter {
                let _ = self.reporter_session.dispatch(Event::Query(query)).await;
            }
        }

    
        // Insert to indexes
        if let Err(e) = self.hash_index.insert(&key, &doc) {
            return Err(SessionResult::Err(e))
        }

        // Insert to tag_index
        self.tag_index.insert(&key, &doc);

        // Insert to range
        self.range_index.insert(&key, &doc);

        // Insert to memory
        self.collection.insert(key, doc);

        Ok(())


    }


    #[inline]
    pub fn insert_content<ContentProvider: GetContent>(&self, key: K, cp: &ContentProvider) 
        -> Option<JoinHandle<()>> {
        match cp.get_content() {
            Some(content) => Some(self.inverted_index.insert(key, content)),
            None => None,
        }
    }

    #[inline]
    pub fn remove_content(&self, key: K, content: String) -> JoinHandle<()> {
        self.inverted_index.remove(key, content)
    }

    /// remove from storage and persist to disk
    #[inline]
    pub async fn remove(&self, key: K) -> Result<(), SessionResult> {
        match self.collection.get(&key) {
            Some(doc) => {
                // remove from hash_index
                self.hash_index.remove(doc.value());

                // remove from tag_index
                self.tag_index.remove(&key, doc.value());

                // remove to range
                self.range_index.remove(&key, doc.value());

            }
            None => return Ok(()),
        }

        self.collection.remove(&key);

        let query = RQuery::<K, Doc>::Remove(key);
    
        match &self.wal_session {
            Some(wal) => {
                // Send to DiskLog
                match wal.log(bincode::serialize(&query).unwrap()).await {
                    Ok(_) => {
                        // if Reporter is on
                        if !self.off_reporter {
                            // Send to Reporter
                            let _ = self.reporter_session.dispatch(Event::Query(query)).await;
                        }

                        Ok(())
                    }
                    Err(e) => Err(e),
                }
            }
            None => {
                // if Reporter is on
                if !self.off_reporter {
                    // Send to Reporter
                    let _ = self.reporter_session.dispatch(Event::Query(query)).await;
                }

                Ok(())
            }
        }

    }

    /// gets documents  
    #[inline]
    pub fn gets(&self, list: Vec<&K>) -> Vec<Ref<K, Doc>> {
        let mut result = Vec::with_capacity(list.len());

        list.iter().for_each(|key| {
            if let Some(r) = self.collection.get(key) {
                result.push(r);
            }
        });

        result
    }

    /// fetch document by range hash_index
    #[inline]
    pub fn range(&self, field_name: &String, from: String, to: String) -> Vec<Ref<K, Doc>> {
        let mut result = Vec::new();

        // collect and distinct keys
        for k in self.range_index.range(field_name, from, to) {
            if let Some(r) = self.collection.get(&k) {
                result.push(r);
            }
        }

        result
    }

    /// lookup by key
    #[inline]
    pub fn lookup(&self, key: &K) -> Option<Ref<K, Doc>> {
        return self.collection.get(key);
    }

    /// lookup by hash_index
    #[inline]
    pub fn lookup_by_index(&self, index_key: &String) -> Option<Ref<K, Doc>> {
        match self.hash_index.lookup(index_key) {
            Some(rf) => {
                self.collection.get(rf.value())
            }
            None => None
        }
    }

    /// lookup by tag
    #[inline]
    pub fn lookup_by_tag(&self, tag: &String) -> Vec<Ref<K, Doc>> {
        match self.tag_index.lookup(tag) {
            Some(rf) => {
                let mut result = Vec::with_capacity(rf.value().len());
                for k in rf.value().iter() {
                    if let Some(kd) = self.collection.get(&k) {
                        result.push(kd);
                    }  
                }
                result
            }
            None => vec![]
        }
    }

    /// search by text
    #[inline]
    pub fn search(&self, text: String) -> Vec<Ref<K, Doc>> {
        let words: Vec<&str> = text.split_whitespace().collect();
        let keys = self.inverted_index.search(words);
        let mut result = Vec::with_capacity(keys.len());
        
        for key in keys {
            if let Some(rd) = self.collection.get(&key) {
                result.push(rd);
            }
        }

        result
    }

    /// return Iter (Safe for mutation)
    #[inline]
    pub fn iter(&self) -> Iter<'_, K, Doc> {
        self.collection.iter()
    }

    /// return Iter (Safe for mutation)
    #[inline]
    pub fn iter_index(&self) -> Iter<'_, String, K> {
        self.hash_index.iter()
    }

    /// return Iter (Safe for mutation)
    #[inline]
    pub fn iter_tags(&self) -> Iter<String, DashSet<K>> {
        self.tag_index.iter()
    }



    /// load storage from disk
    #[inline]
    async fn loader(&self) {
        // when storage just open with Disc Copies option it call loader, else it don't call
        let wal = self.wal_session.as_ref().unwrap();

        let mut page_index = 1;

        loop {
            // Get Page
            let mut logfile = match wal.get_page(page_index).await {
                Ok(lf) => lf,
                Err(sess_res) => {
                    if let SessionResult::Err(status_res) = sess_res {
                        match status_res {
                            StatusResult::LogErr(e) => eprintln!("==> {:?}", e),
                            StatusResult::IoError(e) => eprintln!("==> {:?}", e),
                            StatusResult::Err(e) => eprintln!("==> {:?}", e),
                            _ => {}
                        }
                    }

                    return;
                }
            };

            page_index += 1;

            // Must Call Recover if return Err, remove unwrap()
            let iter = match logfile.iter(..) {
                Ok(iter) => iter,
                Err(e) => {
                    eprintln!("==> {:?}", e);
                    return;
                }
            };

            for qline in iter {
                let query: RQuery<K, Doc> = bincode::deserialize(&qline.unwrap()).unwrap();
                match query {
                    RQuery::Insert(key, doc) => {
                        // Insert to indexes
                        let _ = self.hash_index.insert(&key, &doc);

                        // Insert to tag_index
                        self.tag_index.insert(&key, &doc);

                        // use collection insert to avoid rewrite to log after insert
                        self.collection.insert(key, doc);
                    }
                    RQuery::Remove(key) => {
                        if let Some(r) = self.collection.get(&key) {
                            // remove hash_index
                            self.hash_index.remove( r.value());

                            // remove from tag_index
                            self.tag_index.remove(&key, r.value());

                            self.collection.remove(&key);
                        }
                    }
                }
            }
        }
    }
}

// used for log to disk
#[derive(Serialize, Deserialize, Clone)]
pub enum RQuery<K, Doc> {
    Insert(K, Doc),
    Remove(K),
}

// used for reporting
#[derive(Clone)]
pub enum Event<K, Doc> {
    Query(RQuery<K, Doc>),
    Subscribed(Sender<Event<K, Doc>>), // distributing signal like NodeFail, ....
}
