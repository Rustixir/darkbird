use anymap::AnyMap;
use simple_wal::LogError;
use std::{hash::Hash, collections::HashSet};
use serde::{Serialize, de::DeserializeOwned};

use crate::{Options, document::Document, Storage};

use super::database::Database;



pub struct Schema {
    datastores: AnyMap,
    names: HashSet<String>
}

impl Schema {

    pub fn new() -> Schema {
        Schema { 
            datastores: AnyMap::new(),
            names: HashSet::new()
        }
    }


    pub async fn with_datastore<'a, K, Doc>(mut self, opts: Options<'a>) -> Result<Schema, SchemaError> 
    where
        Doc: Serialize + DeserializeOwned + Clone + Send + 'static + Document,
        K:  Serialize
            + DeserializeOwned
            + PartialOrd
            + Ord
            + PartialEq
            + Eq
            + Hash
            + Clone
            + Send
            + Sync
            + 'static
    {

        if self.names.contains(opts.storage_name) {
            return Err(SchemaError::DatastoreAlreadyExist(opts.storage_name.to_owned()))
        }

        if let Some(_) = self.datastores.get::<Storage<K, Doc>>() {
            return Err(SchemaError::DatastoreAlreadyExist(opts.storage_name.to_owned()))
        }


        match Storage::<K, Doc>::open(opts).await {
            Err(e) => Err(SchemaError::OpenFailed(e)),
            Ok(ds) => {
                self.datastores.insert(ds);
                Ok(self)
            }
        }
    }


    pub fn build(self) -> Database {
        Database::open(self.datastores)
    }

}



#[derive(Debug)]
pub enum SchemaError {
    OpenFailed(LogError),
    DatastoreAlreadyExist(String)
}