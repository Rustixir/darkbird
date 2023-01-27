use std::{collections::{BTreeMap, BTreeSet}, ops::Bound};

use crate::document::Document;
use dashmap::{DashMap, DashSet};
use serde::{de::DeserializeOwned, Serialize};
use std::hash::Hash;

pub struct RangeIndex<K> {
    multi_btree: DashMap<String, BTreeMap<String, DashSet<K>>>,
}

impl<K> RangeIndex<K>
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
        RangeIndex {
            multi_btree: DashMap::new(),
        }
    }

    /// insert entry to tree
    #[inline]
    pub fn insert<Doc>(&self, key: &K, doc: &Doc)
    where
        Doc: Document,
    {
        doc.get_fields()
            .into_iter()
            .for_each(|rf| {
                let val = rf.value;
                match self.multi_btree.get_mut(&rf.name) {
                    Some(mut tree) => match tree.get_mut(&val) {
                        Some(set) => {
                            set.insert(key.clone());
                        }
                        None => {
                            let set = DashSet::new();
                            set.insert(key.clone());
                            tree.value_mut().insert(val, set);
                        }
                    },
                    None => {
                        let mut tree = BTreeMap::new();
                        let set = DashSet::new();
                        set.insert(key.clone());
                        tree.insert(val, set);
                        self.multi_btree.insert(rf.name, tree);
                    }
                }
            });
    }

    /// remove entry from tree
    #[inline]
    pub fn remove<Doc>(&self, key: &K, doc: &Doc)
    where
        Doc: Document,
    {
        doc.get_fields().into_iter().for_each(|rf| {
            if let Some(mut tree) = self.multi_btree.get_mut(&rf.name) {
                if let Some(set) = tree.value_mut().get_mut(&rf.value) {
                    set.insert(key.clone());
                }
            }
        });
    }

    /// remove tree from multi-tree
    #[inline]
    pub fn remove_tree(&self, field_name: &str) {
        self.multi_btree.remove(field_name);
    }


    /// fetch document by range hash_index
    #[inline]
    pub fn range(&self, field_name: &str, from: String, to: String) -> Vec<K> {
        let set = match self.multi_btree.get(field_name) {
            Some(tree) => {
                let mut set_result = BTreeSet::new();

                // collect and distinct keys
                for (_, set) in tree.range((Bound::Included(from), Bound::Excluded(to))) {
                    for k in set.iter() {
                        set_result.insert(k.key().clone());
                    }
                }

                set_result
            }
            None => BTreeSet::new()
        };

        set
        .into_iter()
        .collect::<Vec<K>>()
    }

    
}
