


use std::{hash::Hash, sync::Arc};

use async_trait::async_trait;
use scylla::{Session, SessionBuilder, transport::errors::NewSessionError};


use serde::{Serialize, de::DeserializeOwned};
use tokio_postgres::{NoTls, Client, Error};

use crate::Storage;


pub struct Stop;


#[async_trait]
pub trait Handler<Key, Document, Response> {
    async fn handle(&self, session: &DatabaseSession, key: &Key, document: &Document) -> Result<(), Stop>;
}


pub enum DatabaseName {
    Scylla,
    Postgres
}

pub enum DatabaseSession {
    Scylla(Session),
    Postgres(Client)
}


pub struct Persistent {
    pub db_session: DatabaseSession,
}

impl Persistent {

    pub async fn connect(db_name: DatabaseName, cfg_string: String) -> Result<Persistent, ()> {
        match db_name {
            DatabaseName::Scylla => {
                let persistent = match Persistent::connect_to_scylla(cfg_string).await {
                    Ok(persistent) => persistent,
                    Err(e) => {
                        eprintln!("{}", e.to_string());
                        return Err(())
                    }
                };

                Ok(persistent)
            }
            DatabaseName::Postgres => {
                let persistent = match Persistent::connect_to_postgres(cfg_string).await {
                    Ok(persistent) => persistent,
                    Err(e) => {
                        eprintln!("{}", e.to_string());
                        return Err(())
                    }
                };

                Ok(persistent)
            }
        }
    }

    async fn connect_to_postgres(cfg_string: String) -> Result<Persistent, Error> {
        
        let (client, connection) = match tokio_postgres::connect(&cfg_string, NoTls).await {
            Ok(res) => res,
            Err(e) => return Err(e)
        };

        // The connection object performs the actual communication with the database,
        // so spawn it off to run on its own.
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });

        
        let persist = Persistent {
            db_session: DatabaseSession::Postgres(client)
        };

        Ok(persist)
    }

    async fn connect_to_scylla(uri: String) -> Result<Persistent, NewSessionError> {
        let session =  match SessionBuilder::new().known_node(uri).build().await {
            Ok(session) => {
                session
            }
            Err(e) => return Err(e),
        };
        

        let persist = Persistent {
            db_session: DatabaseSession::Scylla(session)
        };
        Ok(persist)
    }



    pub async fn copy_memtable_to_database<'a, Key, Document, THandler>
           (&self, 
            storage: Arc<Storage<Key, Document>>,
            handler: THandler) 

    where
        Key      : Clone + Serialize + DeserializeOwned + Eq + Hash + Send + 'static,
        Document : Clone + Serialize + DeserializeOwned + Eq + Hash + Send + 'static,
        THandler : Handler<Key, Document, ()>
    {     
        for refi in storage.iter() {
            let key = refi.key();
            let document = refi.value();

            if let Err(Stop) = handler.handle(&self.db_session, key, document).await {
                println!("Stop copying ....");
                return
            }
        }                
    }

}

