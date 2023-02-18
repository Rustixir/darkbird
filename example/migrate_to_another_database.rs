
#[tokio::main]
async fn main() {


    // path can be '.' in storage::open
    let path           = ".";

    let storage_name   = "blackbird";
    let total_page_size = 1000;

    // ** RamCopies ** 
    //     Dont log to disk and after shutdown , loss data

    let stype = StorageType::RamCopies ;

    // ** DiskCopies ** 
    //      whole storage operation is in-memory but log to disk, 
    //       automatic load whole data after restart, avoid any loss data
    //  
    // let stype = StorageType::DiskCopies;

    
    let ops = Options::new(path, storage_name, total_page_size, StorageType::RamCopies, true);


    // Storage is built for high concurrency usage
    // dont need to Mutex / Rwlock, it is safe for shared between thread 
    //
    // create storage and initialize other service (reporter, disk_log)  
    //
    // when calling storage::open, it load whole storage from disk
    //
    
    let s = Arc::new(Storage::<Pid, User>::open(ops).await.unwrap());
    
    let pw = Persistent::connect(DatabaseName::Postgres, "host=localhost user=postgres").await.unwrap();
    
    // Copy Table to database
    pw.copy_memtable_to_database(s.clone(), &WriteToDatabase).await;
    
    
    
}


struct WriteToDatabase;
#[async_trait]
impl Handler<i64, User> for WriteToDatabase {
    async fn handle(&self, session: &DatabaseSession, key: &i64, document: &User) -> Result<(), Stop> {
        if let (DatabaseSession::Postgres(client)) = session {
            let res = client.query("Insert Into tbluser (key, document) values ($1::TEXT, $1::TEXT)", &[key, document.fullname]).await;
            if Err(_) = res {
                return Err(Stop) 
            }
        }
    }
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