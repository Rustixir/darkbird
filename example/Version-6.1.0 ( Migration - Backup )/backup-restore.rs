
use darkbird::{
    document::{self, RangeField},
    Options, Storage, StorageType, page_processor::Sync, backup, RQuery, Schema,
};
use serde_derive::{Deserialize, Serialize};


#[tokio::main]
async fn main() {
    
    let path = ".";
    let storage_name = "blackbird";
    let total_page_size = 1000;
    let stype = StorageType::DiskCopies;
    let vacuum = true;


    
    // Backup:  create full copy from Wal files + DateTime
    backup::<Pid, Profile>(path, storage_name, total_page_size, vacuum).unwrap();
    
    
    // Restore: just set name of backup folder as name 
    let ops = Options::new(path, "blackbird_backup_2023-02-18UTC-08:05:33.988186067", total_page_size, stype, true);


    // Open and load Database
    let db = Schema::new().with_datastore::<Pid, Profile>(ops).await.unwrap().build();
    
    
    let iter = db.iter::<Pid, Profile>().unwrap();
    for kd in iter {
        println!("==> {}-{} {}", kd.key(), kd.value().fullname, kd.value().age);
    }

 
}

type Pid = String;


#[derive(Serialize, Deserialize, Clone, Debug)]
struct Profile {
    fullname: String,
    age: i32
}

impl Profile {
    pub fn new(fullname: &str) -> Self {
        Profile {
            fullname: fullname.to_owned(),
            age: 24
        }
    }
}

impl document::Document for Profile {}

impl document::Indexer for Profile {
    fn extract(&self) -> Vec<String> {
        vec![]
    }
}

impl document::Tags for Profile {
    fn get_tags(&self) -> Vec<String> {
        vec![]
    }
}

impl document::Range for Profile {
    fn get_fields(&self) -> Vec<RangeField> {
        vec![]
    }
}

impl document::MaterializedView for Profile {
    fn filter(&self) -> Option<String> {
        None
    }
}

impl document::FullText for Profile {
    fn get_content(&self) -> Option<String> {
        None
    }
}