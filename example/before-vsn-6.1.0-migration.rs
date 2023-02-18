#[tokio::main]
async fn main() {

    // path must be full path for migration::run
    let path           = "/home/ai/Documents/Rust/blackbird";

    let storage_name   = "blackbird";
    let total_page_size = 1000;



    // if storage exist, next time can call migration::run before calling storage::open,
    //      for update schmea (key, document)

    migration::run(path, 
                   storage_name, 
                   total_page_size, 
                   |rquery: RQuery<Pid, User>| -> RQuery<Pid, User2> {
                        match rquery {
                            RQuery::Insert(pid, doc) => {
                                RQuery::Insert(pid, User2::new(doc.fullname, 23))
                            }
                            RQuery::Remove(pid) => {
                                RQuery::Remove(pid)
                            }
                        }
                    });


    

    let s = Storage::<Pid, User2>::open(path, storage_name, total_page_size, true).await.unwrap();

    
}


type Pid = String;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct User {   
    fullname: String,
}

impl User {
    pub fn new(fullname: &str) -> Self {
        User { fullname: fullname.to_owned() }
    } 
}





#[derive(Serialize, Deserialize, Clone, Debug)]
struct User2 {   
    fullname: String,
    age: i32
}

impl User2 {
    pub fn new(fullname: String, age: i32) -> Self {
        User2 { fullname, age }
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
        None
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


// ========================================

impl document::Document for User2 {}


impl document::Indexer for User2 {
    fn extract(&self) -> Vec<String> {
        vec![]
    }
}

impl document::Tags for User2 {
    fn get_tags(&self) -> Vec<String> {
        vec![]
    }
}

impl document::Range for User2 {
    fn get_fields(&self) -> Vec<RangeField> {
        None
    }
}

impl document::MaterializedView for User2 {
    fn filter(&self) -> Option<String> {
        None
    }
}


impl document::FullText for User2 {
    fn get_content(&self) -> Option<String> {
        None
    }
}