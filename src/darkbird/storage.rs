
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_derive::{Serialize, Deserialize};
use simple_wal::LogError;
use tokio::sync::mpsc::Sender;
use std::hash::Hash;

use dashmap::{DashMap, iter::Iter, mapref::one::Ref};

use super::{disk_log::{DiskLog, Session}, router::{Router, self}, StatusResult, Options, StorageType, indexer::Indexer};

use crate::darkbird::SessionResult;


pub struct Storage<Key, Document: Indexer> {
    
    // DashMap
    engine: DashMap<Key, Document>,

    // Index
    index: DashMap<String, Key>,

    // Wal session
    wal_session: Option<Session>,

    // Reporter session 
    reporter_session: router::Session<Event<Key, Document>>,

    off_reporter: bool
    
}

impl<Key: 'static, Document: 'static> Storage<Key, Document> 
where    
    Key: Serialize + DeserializeOwned + Eq + Hash + Clone + Send,
    Document: Serialize + DeserializeOwned + Clone + Send + Indexer
{
    
    pub async fn open<'a>(ops: Options<'a>) -> Result<Self, LogError> {
            
        if let StorageType::DiskCopies = ops.stype {
            match DiskLog::open(ops.path, ops.storage_name, ops.total_page_size) {
                Err(e) => return Err(e),
                Ok(disklog) => {
    
                    // Run DiskLog 
                    let wal_session = disklog.run_service();
    
                    // Run Reporter
                    let reporter = 
                            Router::<Event<Key, Document>>::new(vec![])
                            .unwrap()
                            .run_service();
    
    
                    // Create Storage
                    let st = Storage { 
                        engine: DashMap::new(),
                        index: DashMap::new(),
                        wal_session: Some(wal_session),
                        reporter_session: reporter,
                        off_reporter: false,
                    };
    
                    // load from disk
                    st.loader().await;                
    
                    return Ok(st)
                }
            }  

        } else {
            
            // Off DiskLog 

            
            // Run Reporter
            let reporter = 
                    Router::<Event<Key, Document>>::new(vec![])
                    .unwrap()
                    .run_service();


            // Create Storage
            let st = Storage { 
                engine: DashMap::new(),
                index: DashMap::new(),
                wal_session: None,
                reporter_session: reporter,
                off_reporter: false,
            };
            
            // loader off
                        
            return Ok(st)

        }

    }

    
    pub fn off_reporter(&mut self) {
        self.off_reporter = true;
    }

    /// subscribe to Reporter
    pub async fn subscribe(&self, sender: Sender<Event<Key, Document>>) -> Result<(), SessionResult>{
        
        if self.off_reporter {
            return Err(SessionResult::Err(StatusResult::ReporterIsOff))
        }
        
        // Send to Reporter        
        let _ = self.reporter_session.dispatch(Event::Subscribed(sender.clone())).await;
        
        self.reporter_session.register(sender).await        
    }


    /// insert to storage and persist to disk
    pub async fn insert(&self, key: Key, doc: Document) -> Result<(), SessionResult> {
        
        match &self.wal_session {
            Some(wal) => {

                let query = RQuery::Insert(key.clone(), doc.clone());

                match wal.log(bincode::serialize(&query).unwrap()).await {
                    Err(e) => Err(e),
                    Ok(_) => {
        
                        // if Reporter is on
                        if !self.off_reporter {
                            // Send to Reporter
                            let _ = self.reporter_session.dispatch(Event::Query(query)).await;
                        }
        

                        // Insert to indexes                        
                        doc
                            .extract()
                            .into_iter()
                            .for_each(|index_key| {
                                self.index.insert(index_key, key.clone());
                            });

                        // Insert to memory
                        self.engine.insert(key, doc);
                        

                        Ok(())
                    }
                } 
            }
            None => {

                // if Reporter is on
                if !self.off_reporter {

                    // Insert to memory
                    let query = RQuery::Insert(key.clone(), doc.clone());
                    
                    // Send to Reporter
                    let _ = self.reporter_session.dispatch(Event::Query(query)).await;
                }


                // Insert to indexes                        
                doc
                    .extract()
                    .into_iter()
                    .for_each(|index_key| {
                        self.index.insert(index_key, key.clone());
                    });
            


                // Insert to memory
                self.engine.insert(key, doc);

                Ok(())
            }
        }       
    }


    /// remove from storage and persist to disk
    pub async fn remove(&self, key: Key) -> Result<(), SessionResult> {


        match self.engine.get(&key) {
            Some(doc) => {                
                doc
                    .value()
                    .extract()
                    .iter()
                    .for_each(|key| {
                        self.index.remove(key);
                    });            
            }
            None => return Ok(()),
        }


        self.engine.remove(&key);

        let query = RQuery::<Key, Document>::Remove(key);

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
    pub fn gets(&self, list: Vec<&Key>) -> Vec<Ref<Key, Document>> {
        let mut result = Vec::with_capacity(list.len());
        
        list
          .iter()
          .for_each(|key| {
              if let Some(r) = self.engine.get(key) {
                result.push(r);
              } 
          });
          
        result
    }


    /// lookup by key
    pub fn lookup(&self, key: &Key) -> Option<Ref<Key, Document>> {
        return self.engine.get(key)
    }


    /// lookup by index
    pub fn lookup_by_index(&self, index_key: &String) -> Option<Ref<Key, Document>> {
        match self.index.get(index_key) {
            Some(r) => {
                return self.engine.get(&r.value())
            }
            None => None,
        }
    }


    /// return Iter (Safe for mutation)
    pub fn iter(&self) -> Iter<'_, Key, Document> {
        self.engine.iter()
    }


    /// return Iter (Safe for mutation)
    pub fn iter_index(&self) -> Iter<'_, String, Key> {
        self.index.iter()
    }



    /// load storage from disk
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

                    return
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

                let query: RQuery<Key, Document> = bincode::deserialize(&qline.unwrap()).unwrap();
                match query {
                    RQuery::Insert(key, doc) => {

                        doc
                        .extract()
                        .into_iter()
                        .for_each(|index_key| {
                            self.index.insert(index_key, key.clone());
                        });

                        // use engine insert to avoid rewrite to log after insert
                        self.engine.insert(key, doc);                                                    
                    }
                    RQuery::Remove(key) => {
                        if let Some(r) = self.engine.get(&key) {
                            r.value()
                            .extract()
                            .iter()
                            .for_each(|index_key| {
                                self.index.remove(index_key);
                            });

                            self.engine.remove(&key);
                        } 
                    }
                }
            }
        }
    }
}



// used for log to disk
#[derive(Serialize, Deserialize, Clone)]
pub enum RQuery<Key, Document> {
    Insert(Key, Document),
    Remove(Key)
}


// used for reporting
#[derive(Clone)]
pub enum Event<Key, Document> {
    Query(RQuery<Key, Document>),
    Subscribed(Sender<Event<Key, Document>>)
    // distributing signal like NodeFail, ....    
}
