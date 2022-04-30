#[tokio::main]
async fn main() {

    // path must be fill path for migration::run
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


    

    let s = Storage::<Pid, User2>::open(path, storage_name, total_page_size).await.unwrap();

    
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
