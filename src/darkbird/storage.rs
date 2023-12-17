use serde::{Serialize, Deserialize, de::DeserializeOwned};
use std::hash::Hash;
use tokio::sync::mpsc::Sender;

use dashmap::{iter::Iter, mapref::one::Ref, DashMap, DashSet};


use super::{
    wal::disk_log::{DiskLog, Session},
    index::{hash::HashIndex, range::RangeIndex, tags::TagIndex, inverted_index::InvertedIndex},
    router::{self, Router},
    Options, StatusResult, StorageType,
};

use crate::{darkbird::SessionResult, document::Document};



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
    wal_session: Session,

    // Reporter session
    reporter_session: router::Session<Event<K, Doc>>,

    off_reporter: bool,

    off_disk: bool
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
    pub async fn open<'a>(ops: Options<'a>) -> Result<Self, String> {
        
        match DiskLog::open(ops.path, ops.storage_name, ops.total_page_size) {
            Err(e) => return Err(e.to_string()),
            Ok(disklog) => {
                // Run DiskLog
                let off_disk = if let StorageType::RamCopies = ops.stype { true } else { false };

                // Run Reporter
                let reporter = Router::<Event<K, Doc>>::new(vec![]).unwrap().run_service();

                // Run disk_log
                let wal_session = disklog.run_service();


                // Create Storage
                let mut st = Storage {
                    collection: DashMap::new(),
                    hash_index: HashIndex::new(),
                    tag_index: TagIndex::new(),
                    range_index: RangeIndex::new(),
                    inverted_index: InvertedIndex::new(),
                    wal_session: wal_session,
                    reporter_session: reporter,
                    off_reporter: ops.off_reporter,
                    off_disk: true
                };


                // load from disk
                if let Err(x) = st.loader().await {
                    if x != "End" {
                        return Err(x);
                    } 
                }


                // because we want loader dont write to disk_log
                st.off_disk = off_disk;

                return Ok(st);
            }
        }

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

        if !self.off_disk || !self.off_reporter {
            let query = RQuery::Insert(key.clone(), doc.clone());

            if !self.off_disk {
                if let Err(e) = self.wal_session.log(bincode::serialize(&query).unwrap()).await {
                    return Err(e);
                }
            }

            if !self.off_reporter {
                let _ = self.reporter_session.dispatch(Event::Query(query)).await;
            }

        }

    
        // Insert to indexes
        if let Err(e) = self.hash_index.insert(&key, &doc) {
            return Err(SessionResult::Err(e))
        }
        

        // Insert to view
        if let Some(view_name) = doc.filter() {
            self.tag_index.insert_view(&view_name, &key)
        }


        // Insert to InvertedIndex
        if let Some(content) = doc.get_content() {
            let _ = self.inverted_index.insert(key.clone(), content).await;
        }


        // Insert to tag_index
        self.tag_index.insert(&key, &doc);


        // Insert to range
        self.range_index.insert(&key, &doc);


        // Insert to memory
        self.collection.insert(key, doc);

        Ok(())


    }

    /// remove from storage and persist to disk
    #[inline]
    pub async fn remove(&self, key: K) -> Result<(), SessionResult> {
        match self.collection.get(&key) {
            Some(doc) => {

                if !self.off_disk || !self.off_reporter {
                    let query = RQuery::<K, Doc>::Remove(key.clone());
        
                    if !self.off_disk {
                        if let Err(e) = self.wal_session.log(bincode::serialize(&query).unwrap()).await {
                            return Err(e);
                        }
                    }
        
                    if !self.off_reporter {
                        let _ = self.reporter_session.dispatch(Event::Query(query)).await;
                    }
                    
                }

                // remove from hash_index
                self.hash_index.remove(doc.value());

                // remove from view
                if let Some(view_name) = doc.filter() {
                    self.tag_index.remove_from_view(&view_name, &key)
                }

                // remove from invertedIndex
                if let Some(content) = doc.value().get_content() {
                    let _ = self.inverted_index.remove(key.clone(), content).await;
                }

                // remove from tag_index
                self.tag_index.remove(&key, doc.value());

                // remove to range
                self.range_index.remove(&key, doc.value());

            }
            None => return Ok(()),
        }

        self.collection.remove(&key);

        Ok(())
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

        /// gets documents  
        #[inline]
        pub fn gets_by_value(&self, list: Vec<K>) -> Vec<Ref<K, Doc>> {
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
    pub fn range(&self, field_name: &str, from: String, to: String) -> Vec<Ref<K, Doc>> {
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
    pub fn lookup_by_index(&self, index_key: &str) -> Option<Ref<K, Doc>> {
        match self.hash_index.lookup(index_key) {
            Some(rf) => {
                self.collection.get(rf.value())
            }
            None => None
        }
    }

    /// lookup by tag
    #[inline]
    pub fn lookup_by_tag(&self, tag: &str) -> Vec<Ref<K, Doc>> {
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

    /// fetch view
    #[inline]
    pub fn fetch_view(&self, view_name: &str) -> Vec<Ref<K, Doc>> {
        match self.tag_index.lookup_view(view_name) {
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

    
    #[inline]
    pub fn collection_len(&self) -> usize {
        self.collection.len()
    }




    /// load storage from disk
    #[inline]
    async fn loader(&self) -> Result<(), String> {
        // when storage just open with Disc Copies option it call loader, else it don't call
        let wal = &self.wal_session;

        let mut page_index = 1;

        loop {
            // Get Page
            let mut logfile = match wal.get_page(page_index).await {
                Ok(lf) => lf,
                Err(sess_res) => {
                    if let SessionResult::Err(e) = sess_res {
                        return Err(e.to_string())
                    }

                    return Err("disk_log closed".to_string())
                }
            };

            page_index += 1;

            // Must Call Recover if return Err, remove unwrap()
            let iter = match logfile.iter(..) {
                Ok(iter) => iter,
                Err(e) => {
                    eprintln!("==> {:?}", e);
                    return Err(e.to_string());
                }
            };

            for qline in iter {
                let bytes = match qline {
                    Ok(ql)  => ql,
                    Err(e) => return Err(e.to_string()),
                };

                let query: RQuery<K, Doc> = match bincode::deserialize(&bytes) {
                    Ok(rq) => rq,
                    Err(e) => {
                        return Err(e.to_string());
                    }
                };

                match query {
                    RQuery::Insert(key, doc) => {                        
                        let _ = self.insert(key, doc).await;
                    }
                    RQuery::Remove(key) => {
                        let _ = self.remove(key).await;
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

impl<K, Doc> RQuery<K, Doc> {
    
    pub fn from_raw(type_id: &'static str, key: K, doc: Option<Doc>) -> RQuery<K, Doc> {
        match type_id {
            RQUERY_INSERT_TYPE => RQuery::Insert(key, doc.unwrap()),
            RQUERY_REMOVE_TYPE => RQuery::Remove(key),
            _ => panic!("failed")
        }
    }

    pub fn into_raw(self) -> (&'static str, K, Option<Doc>) {
        match self {
            RQuery::Insert(k, d) => (RQUERY_INSERT_TYPE, k, Some(d)),
            RQuery::Remove(k) => (RQUERY_REMOVE_TYPE, k, None),
        }
    }

}


pub const RQUERY_INSERT_TYPE: &'static str = "Insert";
pub const RQUERY_REMOVE_TYPE: &'static str = "Remove";





// used for reporting
#[derive(Clone)]
pub enum Event<K, Doc> {
    Query(RQuery<K, Doc>),
    Subscribed(Sender<Event<K, Doc>>), 
}


