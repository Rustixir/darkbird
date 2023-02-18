use darkbird::{Options, Storage, StorageType, document::{self, RangeField}};
use serde_derive::{Deserialize, Serialize};

#[tokio::main]
async fn main() {
    let path = ".";
    let storage_name = "blackbird";
    let total_page_size = 1000;
    let stype = StorageType::DiskCopies;


    let ops = Options::new(path, storage_name, total_page_size, stype, true);
    let storage = Storage::<Pid, User>::open(ops).await.unwrap();


    let id = "1234567".to_string();
    let u = User {
        user: "Danyalmhai".to_string(),
        pass: "652398".to_string(),
        phone: "09370156893".to_string(),
    };

    storage.insert(id, u).await.unwrap();

    

    let res1 = storage.lookup_by_index("Danyalmhai").unwrap();
    let res2 = storage.lookup_by_index("652398").unwrap();
    let res3 = storage.lookup_by_index("09370156893").unwrap();

    

    let is_same_doc = (res1.value() == res2.value()) && (res2.value() == res3.value());
    if is_same_doc {
        println!("they are same Document")
    }

}


type Pid = String;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct User {
    user: String,
    pass: String,
    phone: String,
}


impl document::Document for User {}


impl document::Indexer for User {
    // this example indexing all fields
    fn extract(&self) -> Vec<String> {
        vec![
            self.user.clone(),
            self.pass.clone(),
            self.phone.clone(),
        ]
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