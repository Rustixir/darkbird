use dashmap::{DashMap, DashSet};
use tokio::{spawn, task::JoinHandle};

use std::{hash::Hash, sync::Arc, collections::HashSet};




pub struct InvertedIndex<K> {
    index: Arc<DashMap<String, DashSet<K>>>,
}

impl<K> InvertedIndex<K>
where
    K: PartialOrd
    +  Ord
    +  PartialEq
    +  Eq
    +  Hash 
    +  Clone
    +  Send
    +  Sync
    + 'static
{
    pub fn new() -> Self {
        InvertedIndex { 
            index: Arc::new(DashMap::new()),
        }
    }


    #[inline]
    pub fn insert(&self, key: K, content: String) -> JoinHandle<()> {
        let index = self.index.clone();
        spawn(async move {
            for word in content.split_whitespace() {
                let word = word.to_lowercase() ;
                let key = &key;
                match index.get_mut(&word) {
                    Some(list) => {
                        if let None = list.value().get(&key) {
                            list.value().insert(key.to_owned());
                        }
                    }
                    None => {
                        let list = DashSet::new();
                        list.insert(key.to_owned());
                        index.insert(word, list);
                    }
                }
            }
        })
    }
    #[inline]
    pub fn remove(&self, key: K, content: String) -> JoinHandle<()> {
        let index = self.index.clone();
        spawn(async move {
            for word in content.split_whitespace() {
                let word = word.to_lowercase() ;
                if let Some(list) = index.get_mut(&word) {
                    list.value().remove(&key);
                }
            }
        })
    }
   
   
    #[inline]
    pub fn search(&self, words: Vec<&str>) -> Vec<K> {
        let mut collector = HashSet::new();
        for w in words {
            let keys = self.inner_search(w);
            self.intersect(keys, &mut collector);
        }

        collector.into_iter().collect()
    }

    #[inline] 
    fn intersect(&self, keys: Vec<K>, collector: &mut HashSet<K>) {
        for key in keys {
            if let None = collector.get(&key) {
                collector.insert(key);
            }
        }
    }


    #[inline] 
    fn inner_search(&self, word: &str) -> Vec<K> {
        let word = word.to_lowercase();
        match self.index.get(&word) {
            Some(list) => {
                list
                    .value()
                    .iter()
                    .map(|entry| entry.key().to_owned() )
                    .collect()
            }
            None => {
                vec![]
            }        
        }
    }



}

