use dashmap::{iter::Iter, mapref::one::Ref, DashMap, DashSet};
use serde::{de::DeserializeOwned, Serialize};

use crate::document::Document;
use std::hash::Hash;

pub struct TagIndex<K> {
    pub tags: DashMap<String, DashSet<K>>,
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

    /// insert entry with tags
    #[inline]
    pub fn insert_view(&self, view_name: &str, key: &K) {
        let view_key = self.view_key_maker(view_name);

        match self.tags.get_mut(&view_key) {
            Some(set) => {
                set.value().insert(key.clone());
            }
            None => {
                let set = DashSet::new();
                set.insert(key.clone());
                self.tags.insert(view_key, set);
            }
        }
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

    /// remove entry from view
    #[inline]
    pub fn remove_from_view(&self, view_name: &str, key: &K) {
        let view_key = &self.view_key_maker(view_name);
        if let Some(set) = self.tags.get_mut(view_key) {
            set.value().remove(&key);
        }
    }


    /// remove tag
    #[inline]
    pub fn remove_tag(&self, tag: &str) {
        self.tags.remove(tag);
    }


    /// remove view
    #[inline]
    pub fn remove_view(&self, view_name: &str) {
        let view_key = self.view_key_maker(view_name);
        self.tags.remove(&view_key);
    }



    /// lookup by tag
    #[inline]
    pub fn lookup(&self, tag: &str) -> Option<Ref<String, DashSet<K>>> {
        self.tags.get(tag)
    }

    
    /// lookup by tag
    #[inline]
    pub fn lookup_view(&self, view_name: &str) -> Option<Ref<String, DashSet<K>>> {
        self.tags.get(&self.view_key_maker(view_name))
    }
    
    
    /// get iter
    #[inline]
    pub fn iter(&self) -> Iter<String, DashSet<K>> {
        self.tags.iter()
    }


    #[inline]
    fn view_key_maker(&self, name: &str) -> String {
        format!("__View__{}", name)
    }
        

}
