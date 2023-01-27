use dashmap::{iter::Iter, mapref::one::Ref, DashMap};
use serde::{de::DeserializeOwned, Serialize};

use crate::{document::Document, darkbird::StatusResult};
use std::hash::Hash;


pub struct HashIndex<K> {
    hash: DashMap<String, K>,
}

impl<K> HashIndex<K>
where
    K: Serialize
        + DeserializeOwned
        + PartialOrd
        + Ord
        + PartialEq
        + Eq
        + Hash
        + Clone
        + Send
        + 'static,
{
    pub fn new() -> Self {
        HashIndex {
            hash: DashMap::new(),
        }
    }

    /// insert entry
    #[inline]
    pub fn insert<Doc>(&self, key: &K, doc: &Doc) -> Result<(), StatusResult>
    where
        Doc: Document,
    {
        let index_keys = doc.extract();
        for ik in index_keys.iter() {
            if let Some(_) = self.hash.get(ik) {
                return Err(StatusResult::Duplicate)
            }
        }

        doc.extract().into_iter().for_each(|index_key| {
            self.hash.insert(index_key, key.clone());
        });

        Ok(())
    }

    /// remove entry
    #[inline]
    pub fn remove<Doc>(&self, doc: &Doc)
    where
        Doc: Document,
    {
        doc.extract().iter().for_each(|index_key| {
            self.hash.remove(index_key);
        });
    }

    /// lookup by index_key
    #[inline]
    pub fn lookup(&self, index_key: &str) -> Option<Ref<String, K>>{
        self.hash.get(index_key)
    }

    /// get iter
    #[inline]
    pub fn iter(&self) -> Iter<String, K>{
        self.hash.iter()
    }
}
