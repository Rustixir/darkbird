
use crate::RQuery;

use super::page_processor::{Format, Sync, PageProcessor};


use chrono::Utc;
use std::hash::Hash;
use serde::{de::DeserializeOwned, Serialize};




/// migration 
/// vacuum: reclaims storage occupied by dead RQuery.
pub fn migration<'a, OldKey, OldDoc, NewKey, NewDoc>(
    root: &'a str, 
    source_name: &'a str, 
    source_total_page_size: usize,
    sync_name: Sync<'a>, 
    vacuum: bool, 
    transform: fn(RQuery<OldKey, OldDoc>) -> RQuery<NewKey, NewDoc>) -> Result<(), String>

where
    OldKey: Serialize + DeserializeOwned + Hash + Eq + PartialEq,
    OldDoc: Serialize + DeserializeOwned,
    NewKey: Serialize + DeserializeOwned + Hash + Eq +,
    NewDoc: Serialize + DeserializeOwned
{

    let pp = PageProcessor::<OldKey, OldDoc, NewKey, NewDoc>::new(
        root, 
        source_name,  
        source_total_page_size, 
        sync_name, 
        vacuum, 
        transform);

    pp.start()
}


// take a full copy from wal files with timestamp
pub fn backup<'a, Key, Doc>(
    root: &'a str, 
    name: &'a str, 
    total_page_size: usize, 
    vacuum: bool)  -> Result<(), String> 

where 
    Key: Serialize + DeserializeOwned + Hash + Eq + PartialEq,
    Doc: Serialize + DeserializeOwned,
{

    let dt = Utc::now();
    let time = dt.time().to_string();
    let date = dt.date().to_string();

    let backup_name = format!("{}_backup_{}-{}", name, date, time);


    let pp = PageProcessor::<Key, Doc, Key, Doc>::new(
        root, 
        name, 
        total_page_size, 
        Sync::New(&backup_name), 
        vacuum, 
        |rq| rq);

    pp.start()
}
