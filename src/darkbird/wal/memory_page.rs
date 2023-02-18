use std::collections::HashMap;
use std::hash::Hash;

use tokio::time::Instant;

use crate::RQuery;


pub struct MemoryPage<K: Eq + PartialEq + Hash, Doc> {
    mapper: HashMap<(&'static str, K), (Instant, Option<Doc>)>
}

impl<K, Doc> MemoryPage<K, Doc>  
where 
    K: Eq + PartialEq + Hash
{
    
    pub fn new() -> Self {
        MemoryPage { mapper: HashMap::new() }
    }

    pub fn stash(&mut self, rquery: RQuery<K, Doc>)  {
        let (type_id, key, doc) = rquery.into_raw();
        let time = Instant::now();
        self.mapper.insert((type_id, key), (time, doc));
    }


    pub fn get_page(self) -> Vec<(Instant, RQuery<K, Doc>)> {
        let mut result = Vec::with_capacity(self.mapper.len());
        for ((type_id, key), (instant, doc)) in self.mapper {
            result.push((instant, RQuery::from_raw(type_id, key, doc)));
        }

        result.sort_by(|(a, _), (b, _)| a.cmp(b));
        result
    } 
}