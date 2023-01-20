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
    let ops = Options::new(path, storage_name, total_page_size, StorageType::RamCopies);

    let storage = Storage::<Pid, User>::open(ops).await.unwrap();

    let key = "+98 9370156893".to_string();
    let doc = User::new("DanyalMh");
    

    // insert spawn future to progress for processing
    // return joinHandle and if you want search after insert, always await on this
    let _ = storage.insert_content(key.clone(), &doc).unwrap().await;
    
    storage.insert(key.clone(), doc.clone()).await.unwrap();


    let result = storage.search(String::from("Is AMazing"));

    if result[0].value().fullname.eq("DanyalMh") {
        println!("find !!");

        // remove spawn future to progress for processingf
        // return joinHandle 

        // use doc.get_content to get all text to remove complete from storage
        let _ = storage.remove_content(key, doc.get_content().unwrap()).await;
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


// GetContent must impl just, when using storage.insert_content(...) 
impl GetContent for User {
    fn get_content(&self) -> Option<String> {
        Some(self.desc.clone())
    }
}