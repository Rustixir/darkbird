use dashmap::{iter::Iter, mapref::one::Ref, DashMap, DashSet};
use serde::{de::DeserializeOwned, Serialize};

use crate::document::Document;
use std::hash::Hash;

pub struct TagIndex<K> {
    tags: DashMap<String, DashSet<K>>,
}

impl<K> TagIndex<K>
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
        TagIndex {
            tags: DashMap::new(),
        }
    }

    /// insert entry with tags
    #[inline]
    pub fn insert<Doc>(&self, key: &K, doc: &Doc)
    where
        Doc: Document,
    {
        doc.get_tags()
            .into_iter()
            .for_each(|index_key| match self.tags.get_mut(&index_key) {
                Some(set) => {
                    set.value().insert(key.clone());
                }
                None => {
                    let set = DashSet::new();
                    set.insert(key.clone());
                    self.tags.insert(index_key, set);
                }
            });
    }

    /// remove entry from tags
    #[inline]
    pub fn remove<Doc>(&self, key: &K, doc: &Doc)
    where
        Doc: Document,
    {
        doc.get_tags().into_iter().for_each(|index_key| {
            if let Some(set) = self.tags.get_mut(&index_key) {
                set.value().remove(&key);
            }
        });
    }

    /// remove tag
    #[inline]
    pub fn remove_tag(&self, tag: &String) {
        self.tags.remove(tag);
    }

    /// lookup by tag
    #[inline]
    pub fn lookup(&self, tag: &String) -> Option<Ref<String, DashSet<K>>> {
        self.tags.get(tag)
    }

    /// get iter
    #[inline]
    pub fn iter(&self) -> Iter<String, DashSet<K>> {
        self.tags.iter()
    }
}
