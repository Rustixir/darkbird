use darkbird::{document::{self, RangeField}, Options, Storage, StorageType};
use serde_derive::{Deserialize, Serialize};

#[tokio::main]
async fn main() {
    let path = ".";
    let storage_name = "blackbird";
    let total_page_size = 1000;
    let stype = StorageType::DiskCopies;

    let ops = Options::new(path, storage_name, total_page_size, stype, true);
    let storage = Storage::<Pid, User>::open(ops).await.unwrap();

    for num in 1..=5 {
        let id = format!("1234567{}", (num * 10) + 1);
        let u = User {
            user: "Danyalmhai".to_string(),
            pass: "652398".to_string(),
            phone: "09370156893".to_string(),
            salary: 100 * num,
            age: num % 24
        };

        storage.insert(id, u).await.unwrap();

        let id = format!("1234567{}", (num * 10) + 2);
        let u = User {
            user: "Danyalmhai".to_string(),
            pass: "652398".to_string(),
            phone: "09370156893".to_string(),
            salary: (100 * num) + 2,
            age: num % 24
        };

        storage.insert(id, u).await.unwrap();

        let id = format!("1234567{}", (num * 10) + 3);
        let u = User {
            user: "Danyalmhai".to_string(),
            pass: "652398".to_string(),
            phone: "09370156893".to_string(),
            salary: (100 * num) + 3,
            age: num % 24
        };

        storage.insert(id, u).await.unwrap();
    }

    let field = "salary".to_string();

    for refkd in storage.range(&field, format!("200"), format!("300")) {
        println!("==> {} - {}", refkd.key(), refkd.value().salary);
    }
}

type Pid = String;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct User {
    user: String,
    pass: String,
    phone: String,
    salary: i32,
    age: i32,
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
        
        let age = "age".to_string();
        let salary = "salary".to_string();

        let vage = format!("{}", self.age);
        let vsalary = format!("{}", self.salary);        
        

        vec![
            RangeField{name: age,    value: vage},
            RangeField{name: salary, value: vsalary}
        ]
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
