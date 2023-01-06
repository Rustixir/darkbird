

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

    let ops = Options::new(path, storage_name, total_page_size, stype);
    let storage = Storage::<Pid, User>::open(ops).await.unwrap();

    for num in 0..20 {
        let id = format!("1234567{}", num);
        let u = User {
            name: "Danyalmhai".to_string(),
            work_at: if num > 10 {
                Company::Uber
            } else {
                Company::Instagram
            },
        };

        storage.insert(id, u).await.unwrap();
    }

    let result = storage.lookup_by_tag(&Company::Uber.to_string());
    for emp in result.iter() {
        match emp.value().work_at {
            Company::Uber => {
                println!("==> {:?} work at Uber", emp.value().name);                
            }
            _ => panic!("unexcepted")
        }
    }
}

type Pid = String;

#[derive(Serialize, Deserialize, Clone, Debug)]
enum Company {
    Instagram,
    Uber,
}
impl ToString for Company {
    fn to_string(&self) -> String {
        match self {
            Company::Instagram => "Instagram".to_owned(),
            Company::Uber => "Uber".to_owned(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct User {
    name: String,
    work_at: Company,
}

impl document::Document for User {}

impl document::Indexer for User {
    fn extract(&self) -> Vec<String> {
        vec![]
    }
}

impl document::Tags for User {
    fn get_tags(&self) -> Vec<String> {
        vec![self.work_at.to_string()]
    }
}

impl document::Range for User {
    fn get_fields(&self) -> Vec<RangeField> {
        vec![]
    }
}
