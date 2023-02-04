use darkbird::{StorageType, Options, Schema, document};
use serde_derive::{Serialize, Deserialize};


fn factory_option(name: &str) -> Options {
    Options::new(".", 
        name,
        1000, 
        StorageType::RamCopies, 
        true
    )
}


#[tokio::main]
async fn main() {
    
    
    let db = Schema::new()
            .with_datastore::<String, Account>(factory_option("Account")).await.unwrap()
            .with_datastore::<i32, AuthData>(factory_option("AuthData")).await.unwrap()
            .build();

    

    let acc_key = "D12".to_owned();
    let account = Account {
        typ: String::from("Admin"),
        funame: String::from("DanyalMh")
    };


    let auth = AuthData{
        access_token: String::from("112azsdaw22361"), 
        refresh_token: String::from("148za018a1a1")
    };


    // insert to Auth Datastore
    db.insert(10, auth).await.unwrap();


    // insert to Account Datastore
    db.insert(acc_key, account).await.unwrap();

    

    let res = db.lookup::<i32, AuthData>(&10).unwrap();
    if let Some(r) = res {
        println!("==> {}", r.value().access_token);
    }
    
    
}




#[derive(Clone, Serialize, Deserialize)]
struct Account {
    typ: String,
    funame: String
}

impl document::Document for Account {}

impl document::Indexer for Account {
    fn extract(&self) -> Vec<String> { vec![] }
}

impl document::Tags for Account {
    fn get_tags(&self) -> Vec<String> { vec![] }
}

impl document::Range for Account {
    fn get_fields(&self) -> Vec<document::RangeField> { vec![] }
}

impl document::MaterializedView for Account {
    fn filter(&self) -> Option<String> { None }
}

impl document::FullText for Account {
    fn get_content(&self) -> Option<String> {
        None
    }
}

// ==========================================

#[derive(Clone, Serialize, Deserialize)]
struct AuthData {
    refresh_token: String,
    access_token: String
}

impl document::Document for AuthData {}

impl document::Indexer for AuthData {
    fn extract(&self) -> Vec<String> { vec![] }
}

impl document::Tags for AuthData {
    fn get_tags(&self) -> Vec<String> { vec![] }
}

impl document::Range for AuthData {
    fn get_fields(&self) -> Vec<document::RangeField> { vec![] }
}

impl document::MaterializedView for AuthData {
    fn filter(&self) -> Option<String> { None }
}

impl document::FullText for AuthData {
    fn get_content(&self) -> Option<String> {
        None
    }
}