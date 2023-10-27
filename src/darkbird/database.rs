use anymap::AnyMap;
use dashmap::{mapref::one::Ref, iter::Iter, DashSet};
use tokio::sync::mpsc::Sender;
use std::{hash::Hash, sync::Arc, time::Duration};
use serde::{de::DeserializeOwned, Serialize};

use crate::{Storage, document::Document, Event, VecStorage, Vector};

use super::{SessionResult, storage_redis::RedisStorage, vector::VectorId};



pub struct Database {
    datastores: AnyMap
}

impl Database {
    

    pub fn open(datastores: AnyMap) -> Database {
        Database { datastores }
    }


    #[inline]        
    pub async fn vec_subscribe(&self, sender: Sender<Event<VectorId, Vector>>) -> Result<(), SessionResult> {
        match self.datastores.get::<VecStorage>() {
            None => Err(SessionResult::DataStoreNotFound),
            Some(datastore) => {
                datastore.subscribe(sender).await
            }
        }
    }

    #[inline]        
    pub async fn vec_insert(&self, vector_id: VectorId, vector: Vec<f32>) -> Result<(), SessionResult> {
        match self.datastores.get::<VecStorage>() {
            None => Err(SessionResult::DataStoreNotFound),
            Some(datastore) => {
                datastore.insert(vector_id, vector).await
            }
        }
    }

    #[inline]        
    pub async fn vec_insert_with_uuid(&self, vector: Vec<f32>) -> Result<(), SessionResult> {
        match self.datastores.get::<VecStorage>() {
            None => Err(SessionResult::DataStoreNotFound),
            Some(datastore) => {
                datastore.insert_with_uuid(vector).await
            }
        }
    }

    #[inline]        
    pub async fn vec_remove(&self, vector_id: VectorId) -> Result<(), SessionResult> {
        match self.datastores.get::<VecStorage>() {
            None => Err(SessionResult::DataStoreNotFound),
            Some(datastore) => {
                datastore.remove(vector_id).await
            }
        }
    }

    
    #[inline]        
    pub fn vec_gets(&self, list: Vec<VectorId>) -> Result<Vec<(String, Vector)>, SessionResult> {
        match self.datastores.get::<VecStorage>() {
            None => Err(SessionResult::DataStoreNotFound),
            Some(datastore) => {
                let res = datastore.gets(list);
                Ok(res)
            }
        }
    }


    #[inline]        
    pub fn vec_lookup(&self, vector_id: &VectorId) -> Result<Option<(VectorId, Vector)>, SessionResult> {
        match self.datastores.get::<VecStorage>() {
            None => Err(SessionResult::DataStoreNotFound),
            Some(datastore) => {
                let res = datastore.lookup(vector_id);
                Ok(res)
            }
        }
    }

    /// // Define a query vector
    /// let query_vector = vec![2.0, 3.0, 4.0];
    ///
    /// // Find the 2 nearest neighbors to the query vector
    /// let nearest_neighbors = db.k_nearest_neighbors(&query_vector, 2);
    /// assert_eq!(nearest_neighbors, vec![
    ///     ("vector1".to_string(), vec![1.0, 2.0, 3.0]),
    ///     ("vector2".to_string(), vec![4.0, 5.0, 6.0])
    /// ]);
    #[inline]        
    pub fn vec_k_nearest_neighbors(&self, query: &Vector, k: usize) -> Result<Vec<(VectorId, Vector)>, SessionResult> {
        match self.datastores.get::<VecStorage>() {
            None => Err(SessionResult::DataStoreNotFound),
            Some(datastore) => {
                let res = datastore.k_nearest_neighbors(query, k);
                Ok(res)
            }
        }
    }

    #[inline]        
    pub fn vec_addition(&self, vec_id1: &str, vec_id2: &str) -> Result<Option<Vector>, SessionResult> {
        match self.datastores.get::<VecStorage>() {
            None => Err(SessionResult::DataStoreNotFound),
            Some(datastore) => {
                let res = datastore.vector_addition(vec_id1, vec_id2);
                Ok(res)
            }
        }
    }

    /// Performs scalar multiplication of a vector stored in the storage.
    #[inline]        
    pub fn vec_scaling(&self, vec_id: &str, scalar: f32) -> Result<Option<Vector>, SessionResult> {
        match self.datastores.get::<VecStorage>() {
            None => Err(SessionResult::DataStoreNotFound),
            Some(datastore) => {
                let res = datastore.vector_scaling(vec_id, scalar);
                Ok(res)
            }
        }
    }


    /// Performs scalar multiplication of a vector stored in the storage.
    #[inline]        
    pub fn vec_subtraction(&self, vec_id: &str, scalar: f32) -> Result<Option<Vector>, SessionResult> {
        match self.datastores.get::<VecStorage>() {
            None => Err(SessionResult::DataStoreNotFound),
            Some(datastore) => {
                let res = datastore.vector_scaling(vec_id, scalar);
                Ok(res)
            }
        }
    }
    
    



    #[inline]        
    pub async fn subscribe<K, Doc>(&self, sender: Sender<Event<K, Doc>>) -> Result<(), SessionResult> 
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
        match self.datastores.get::<Storage<K, Doc>>() {
            None => Err(SessionResult::DataStoreNotFound),
            Some(datastore) => {
                datastore.subscribe(sender).await
            }
        }
    }

    #[inline]        
    pub async fn insert<K, Doc>(&self, key: K, doc: Doc) -> Result<(), SessionResult>
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
        match self.datastores.get::<Storage<K, Doc>>() {
            None => Err(SessionResult::DataStoreNotFound),
            Some(datastore) => {
                datastore.insert(key, doc).await
            }
        }
    }

    #[inline]        
    pub async fn remove<K, Doc>(&self, key: K) -> Result<(), SessionResult>
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
        match self.datastores.get::<Storage<K, Doc>>() {
            None => Err(SessionResult::DataStoreNotFound),
            Some(datastore) => {
                datastore.remove(key).await
            }
        }
    }


    
    #[inline]        
    pub fn gets<'a, K, Doc>(&self, list: Vec<&K>) -> Result<Vec<Ref<K, Doc>>, SessionResult>
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
        match self.datastores.get::<Storage<K, Doc>>() {
            None => Err(SessionResult::DataStoreNotFound),
            Some(datastore) => {
                let res = datastore.gets(list);
                Ok(res)
            }
        }
    }


    #[inline]        
    pub fn range<K, Doc>(&self, field_name: &str, from: String, to: String) -> Result<Vec<Ref<K, Doc>>, SessionResult>
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
        match self.datastores.get::<Storage<K, Doc>>() {
            None => Err(SessionResult::DataStoreNotFound),
            Some(datastore) => {
                let res = datastore.range(field_name, from, to);
                Ok(res)
            }
        }
    }



    #[inline]        
    pub fn lookup<K, Doc>(&self, key: &K) -> Result<Option<Ref<K, Doc>>, SessionResult> 
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
        match self.datastores.get::<Storage<K, Doc>>() {
            None => Err(SessionResult::DataStoreNotFound),
            Some(datastore) => {
                let res = datastore.lookup(key);
                Ok(res)
            }
        }
    }


    #[inline]        
    pub fn lookup_by_index<K, Doc>(&self, index_key: &str) -> Result<Option<Ref<K, Doc>>, SessionResult>
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
        match self.datastores.get::<Storage<K, Doc>>() {
            None => Err(SessionResult::DataStoreNotFound),
            Some(datastore) => {
                let res = datastore.lookup_by_index(index_key);
                Ok(res)
            }
        }
    }



    #[inline]        
    pub fn lookup_by_tag<K, Doc>(&self, tag: &str) -> Result<Vec<Ref<K, Doc>>, SessionResult>
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
        match self.datastores.get::<Storage<K, Doc>>() {
            None => Err(SessionResult::DataStoreNotFound),
            Some(datastore) => {
                let res = datastore.lookup_by_tag(tag);
                Ok(res)
            }
        }
    }



    #[inline]        
    pub fn fetch_view<K, Doc>(&self, view_name: &str) -> Result<Vec<Ref<K, Doc>>, SessionResult>
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
        match self.datastores.get::<Storage<K, Doc>>() {
            None => Err(SessionResult::DataStoreNotFound),
            Some(datastore) => {
                let res = datastore.fetch_view(view_name);
                Ok(res)
            }
        }
    }



    #[inline]        
    pub fn search<K, Doc>(&self, text: String) -> Result<Vec<Ref<K, Doc>>, SessionResult>
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
        match self.datastores.get::<Storage<K, Doc>>() {
            None => Err(SessionResult::DataStoreNotFound),
            Some(datastore) => {
                let res = datastore.search(text);
                Ok(res)
            }
        }
    }



    #[inline]        
    pub fn iter<K, Doc>(&self) -> Result<Iter<'_, K, Doc>, SessionResult>
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
        match self.datastores.get::<Storage<K, Doc>>() {
            None => Err(SessionResult::DataStoreNotFound),
            Some(datastore) => {
                let res = datastore.iter();
                Ok(res)
            }
        }
    }


    #[inline]        
    pub fn iter_index<K, Doc>(&self) -> Result<Iter<String, K>, SessionResult>
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
        match self.datastores.get::<Storage<K, Doc>>() {
            None => Err(SessionResult::DataStoreNotFound),
            Some(datastore) => {
                let res = datastore.iter_index();
                Ok(res)
            }
        }
    }


    #[inline]        
    pub fn iter_tags<K, Doc>(&self) -> Result<Iter<String, DashSet<K>>, SessionResult>
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
        match self.datastores.get::<Storage<K, Doc>>() {
            None => Err(SessionResult::DataStoreNotFound),
            Some(datastore) => {
                let res = datastore.iter_tags();
                Ok(res)
            }
        }
    }




    /// Just for redisstore engine
    #[inline]
    pub fn set<K, Doc>(&self, key: K, value: Doc, expire: Option<Duration>) -> Result<(), SessionResult>
    where
        Doc: Clone + Send + Sync + 'static,
        K:  Clone
            + PartialOrd
            + Ord
            + PartialEq
            + Eq
            + Hash
            + Send
            + 'static
    {
        match self.datastores.get::<RedisStorage<K, Doc>>() {
            None => Err(SessionResult::DataStoreNotFound),
            Some(datastore) => {
                datastore.set(key, value, expire);
                Ok(())
            }
        }
    }


    /// Just for redisstore engine
    #[inline]
    pub fn get<K, Doc>(&self, key: &K) -> Result<Option<Arc<Doc>>, SessionResult>
    where
        Doc: Clone + Send + Sync + 'static,
        K:  Clone
            + PartialOrd
            + Ord
            + PartialEq
            + Eq
            + Hash
            + Send
            + 'static
    {
        match self.datastores.get::<RedisStorage<K, Doc>>() {
            None => Err(SessionResult::DataStoreNotFound),
            Some(datastore) => {
                Ok(datastore.get(key))
            }
        }
    }


    /// Just for redisstore engine
    #[inline]
    pub fn del<K, Doc>(&self, key: &K) -> Result<(), SessionResult>
    where
        Doc: Clone + Send + Sync + 'static,
        K:  Clone
            + PartialOrd
            + Ord
            + PartialEq
            + Eq
            + Hash
            + Send
            + 'static
    {
        match self.datastores.get::<RedisStorage<K, Doc>>() {
            None => Err(SessionResult::DataStoreNotFound),
            Some(datastore) => {
                Ok(datastore.del(key))
            }
        }
    }



    /// Just for redisstore engine
    #[inline]
    pub fn set_nx<K, Doc>(&self, key: K, value: Doc, expire: Option<Duration>) -> Result<bool, SessionResult>
    where
        Doc: Clone + Send + Sync + 'static,
        K:  Clone
            + PartialOrd
            + Ord
            + PartialEq
            + Eq
            + Hash
            + Send
            + 'static
    {
        match self.datastores.get::<RedisStorage<K, Doc>>() {
            None => Err(SessionResult::DataStoreNotFound),
            Some(datastore) => {
                Ok(datastore.set_nx(key, value, expire))
            }
        }
    }



    



}