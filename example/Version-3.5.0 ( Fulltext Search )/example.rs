use darkbird::{
    document::{self, RangeField, GetContent},
    Options, Storage, StorageType,
};
use serde_derive::{Deserialize, Serialize};

#[tokio::main]
async fn main() {
    
    let path = ".";
    let storage_name = "blackbird";
    let total_page_size = 1000;
    let ops = Options::new(path, storage_name, total_page_size, StorageType::RamCopies, true);

    let storage = Storage::<Pid, User>::open(ops).await.unwrap();

    let key = "+98 9370156893".to_string();
    let doc = User::new("DanyalMh");
    

    storage.insert(key, doc).await.unwrap();


    let result = storage.search(String::from("Is AMazing")).await;

    
    if result[0].value().fullname.eq("DanyalMh") {
        println!("find !!");

        // remove spawn future to progress for processingf
        // return joinHandle 

        // use doc.get_content to get all text to remove complete from storage
        // if you make sure document exist in storage call unwrap unless if document 
        // not exit in storage check result
        let _ = storage.remove(key).unwrap().await;
    }


    let result = storage.search(String::from("Is AMazing"));
    if result.len() == 0 {
        println!("removed !!");
    } else {
        panic!("isn't removed already exist.")
    }


}

type Pid = String;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct User {
    fullname: String,
    desc: String
}

impl User {
    pub fn new(fullname: &str) -> Self {
        User {
            fullname: fullname.to_owned(),
            desc: "Rust is amazing".to_owned()
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
        Some(self.desc.clone())
    }
}