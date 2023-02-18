

use darkbird::{
    document::{self, RangeField},
    Options, Storage, StorageType,
};
use serde_derive::{Deserialize, Serialize};

#[tokio::main]
async fn main() {
    let path = ".";
    let storage_name = "blackbird";
    let total_page_size = 1000;
    let stype = StorageType::DiskCopies;

    let ops = Options::new(path, storage_name, total_page_size, stype, true);
    let storage = Storage::<Pid, User>::open(ops).await.unwrap();

    // Generate
    for num in 0..20 {
        let id = format!("1234567{}", num);
        let u = User {
            name: "Danyalmhai".to_string(),
            age: num % 24,
            account_type: if (num% 25) > 18 { AccountType::Admin } else { AccountType::Guest },
            access_level: if (num% 25) >= 23 { 1 } else { 3 }
        };

        storage.insert(id, u).await.unwrap();
    }

    let super_admins = storage.fetch_view("Super Admin");
    let admins = storage.fetch_view("Admin");

}

type Pid = String;


#[derive(Serialize, Deserialize, Clone, Debug)]
enum AccountType {
    Admin,
    Guest
}


#[derive(Serialize, Deserialize, Clone, Debug)]
struct User {
    name: String,
    age: i32,
    account_type: AccountType,
    access_level: u8
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

    // Return None or Some(view_name)
    fn filter(&self) -> Option<String> {
        match (&self.account_type, self.access_level) {
            
            (AccountType::Admin, 1, )  => Some(format!("Super Admin")),
            (AccountType::Admin, _, )  => Some(format!("Admin")),
            (AccountType::Guest, _, )  => None,                 
        }
    }
}


impl document::FullText for User {
    fn get_content(&self) -> Option<String> {
        None
    }
}