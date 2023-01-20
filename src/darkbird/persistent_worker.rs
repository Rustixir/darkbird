use async_trait::async_trait;
use scylla::{transport::errors::NewSessionError, Session, SessionBuilder};
use serde::{de::DeserializeOwned, Serialize};
use std::{hash::Hash, sync::Arc};

use tokio_postgres::{Client, Error, NoTls};

use crate::{document::Document, Storage};

use super::SessionResult;

pub struct Stop;

#[async_trait]
pub trait Setter<K, Doc> {
    async fn init(&self, session: &DatabaseSession);
    async fn handle_setter(
        &self,
        session: &DatabaseSession,
        key: &K,
        document: &Doc,
    ) -> Result<(), Stop>;
}

#[async_trait]
pub trait Getter<K, Doc> {
    async fn handle_getter(&self, session: &DatabaseSession) -> Vec<(K, Doc)>;
}

pub enum DatabaseName {
    Scylla,
    Postgres,
}

pub enum DatabaseSession {
    Scylla(Session),
    Postgres(Client),
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
                        return Err(());
                    }
                };

                Ok(persistent)
            }
            DatabaseName::Postgres => {
                let persistent = match Persistent::connect_to_postgres(cfg_string).await {
                    Ok(persistent) => persistent,
                    Err(e) => {
                        eprintln!("{}", e.to_string());
                        return Err(());
                    }
                };

                Ok(persistent)
            }
        }
    }

    async fn connect_to_postgres(cfg_string: String) -> Result<Persistent, Error> {
        let (client, connection) = match tokio_postgres::connect(&cfg_string, NoTls).await {
            Ok(res) => res,
            Err(e) => return Err(e),
        };

        // The connection object performs the actual communication with the database,
        // so spawn it off to run on its own.
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });

        let persist = Persistent {
            db_session: DatabaseSession::Postgres(client),
        };

        Ok(persist)
    }

    async fn connect_to_scylla(uri: String) -> Result<Persistent, NewSessionError> {
        let session = match SessionBuilder::new().known_node(uri).build().await {
            Ok(session) => session,
            Err(e) => return Err(e),
        };

        let persist = Persistent {
            db_session: DatabaseSession::Scylla(session),
        };
        Ok(persist)
    }

    pub async fn copy_memtable_to_database<K, Doc, THandler>(
        &self,
        storage: Arc<Storage<K, Doc>>,
        handler: &THandler,
    ) where
        THandler: Setter<K, Doc>,
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
        for refi in storage.iter() {
            let key = refi.key();
            let document = refi.value();

            if let Err(Stop) = handler.handle_setter(&self.db_session, key, document).await {
                println!("Stop copying ....");
                return;
            }
        }
    }

    pub async fn load_memtable_from_database<K, Doc, THandler>(
        &self,
        storage: Arc<Storage<K, Doc>>,
        handler: &THandler,
    ) -> Result<(), SessionResult>
    where
        THandler: Getter<K, Doc>,
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
        // Call Getter
        let list = handler.handle_getter(&self.db_session).await;

        // Fill Memtable
        for (key, document) in list.into_iter() {
            if let Err(e) = storage.insert(key, document).await {
                return Err(e);
            }
        }

        return Ok(());
    }
}
