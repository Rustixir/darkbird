use anymap::AnyMap;
use std::{hash::Hash, collections::HashSet};
use serde::{Serialize, de::DeserializeOwned};

use crate::{Options, document::Document, Storage, VecStorage};

use super::{database::Database, storage_redis::RedisStorage};



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
        Doc: Serialize + DeserializeOwned + Clone + Sync + Send + 'static + Document,
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
            Err(e) => Err(SchemaError::Err(e)),
            Ok(ds) => {
                self.datastores.insert(ds);
                Ok(self)
            }
        }
        
    }


    pub async fn with_vecstore<'a>(mut self, opts: Options<'a>) -> Result<Schema, SchemaError> {

        if self.names.contains(opts.storage_name) {
            return Err(SchemaError::DatastoreAlreadyExist(opts.storage_name.to_owned()))
        }

        if let Some(_) = self.datastores.get::<VecStorage>() {
            return Err(SchemaError::DatastoreAlreadyExist(opts.storage_name.to_owned()))
        }

        match VecStorage::open(opts).await {
            Err(e) => Err(SchemaError::Err(e)),
            Ok(ds) => {
                self.datastores.insert(ds);
                Ok(self)
            }
        }
        
    }



    pub async fn with_redisstore<K, Doc>(mut self, storage_name: &str) -> Result<Schema, SchemaError> 
    where
        Doc: Clone + Sync + Send + 'static,
        K:  
            PartialOrd
            + Ord
            + PartialEq
            + Eq
            + Hash
            + Clone
            + Send
            + Sync
            + 'static
    {

        if self.names.contains(storage_name) {
            return Err(SchemaError::DatastoreAlreadyExist(storage_name.to_owned()))
        }

        if let Some(_) = self.datastores.get::<RedisStorage<K, Doc>>() {
            return Err(SchemaError::DatastoreAlreadyExist(storage_name.to_owned()))
        }

        self.datastores.insert(RedisStorage::<K, Doc>::new());
        Ok(self)
        
    }




    pub fn build(self) -> Database {
        Database::open(self.datastores)
    }

}



#[derive(Debug)]
pub enum SchemaError {
    DatastoreAlreadyExist(String),
    Err(String)
}