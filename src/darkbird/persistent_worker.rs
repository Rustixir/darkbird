


use std::{hash::Hash, pin::Pin, future::Future};

use scylla::{Session, SessionBuilder, transport::errors::NewSessionError};


use serde::{Serialize, de::DeserializeOwned};
use tokio_postgres::{NoTls, Error, Client};

use crate::Storage;


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



    pub async fn copy_memtable_to_database<'a, Key, Document>
           (&self, 
            storage: Storage<Key, Document>,
            handler: fn(&DatabaseSession, &Key, &Document) -> Pin<Box<dyn Future<Output=()> + 'a>>) 

    where
        Key: Clone + Serialize + DeserializeOwned + Eq + Hash + Send + 'static,
        Document: Clone + Serialize + DeserializeOwned + Eq + Hash + Send + 'static
    {     
        for refi in storage.iter() {
            let key = refi.key();
            let document = refi.value();

            handler(&self.db_session, key, document).await;
        }                
    }

}

