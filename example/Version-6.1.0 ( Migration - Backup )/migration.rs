
use darkbird::{
    document::{self, RangeField},
    Options, Storage, StorageType, migration, page_processor::Sync, backup, RQuery, Schema,
};
use serde_derive::{Deserialize, Serialize};


#[tokio::main]
async fn main() {
    
    let path = ".";
    let storage_name = "blackbird";
    let total_page_size = 1000;
    let stype = StorageType::DiskCopies;

    let ops = Options::new(path, storage_name, total_page_size, stype, true);
    
    
    // Migration must call before open database
    migration::<Pid, User, Pid, Profile>(
        path, 
        storage_name, 
        total_page_size, 
        Sync::Overwrite, 
        true, |rq| 
        {
            match rq {
                RQuery::Insert(k, u) => {

                    let p = Profile {
                        fullname: u.fullname,
                        age: 25
                    };
                    RQuery::Insert(k, p)  
                      
                }
                RQuery::Remove(k) => RQuery::Remove(k)
            }
        }
    ).unwrap();

    
    // create schema with new document model
    let db = Schema::new().with_datastore::<Pid, Profile>(ops).await.unwrap().build();
    
    
    let iter = db.iter::<Pid, Profile>().unwrap();


    for kd in iter {
        println!("==> {}-{} {}", kd.key(), kd.value().fullname, kd.value().age);
    }

 
}

type Pid = String;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct User {
    fullname: String,
}

impl User {
    pub fn new(fullname: &str) -> Self {
        User {
            fullname: fullname.to_owned(),
        }
    }
}

impl document::Document for User {}

impl document::Indexer for User {
    fn extract(&self) -> Vec<String> {
        vec![]
    }
}

impl document::Tags for User {
    fn get_tags(&self) -> Vec<String> {
        vec![]
    }
}

impl document::Range for User {
    fn get_fields(&self) -> Vec<RangeField> {
        vec![]
    }
}

impl document::MaterializedView for User {
    fn filter(&self) -> Option<String> {
        None
    }
}


impl document::FullText for User {
    fn get_content(&self) -> Option<String> {
        None
    }
}




// / ==============================================


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