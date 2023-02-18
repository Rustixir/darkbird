
#[tokio::main]
async fn main() {

    let path           = ".";
    let storage_name   = "blackbird";
    let total_page_size = 1000;

    let stype = StorageType::RamCopies ;
    let ops = Options::new(path, storage_name, total_page_size, StorageType::RamCopies, true);


    // **** Performance Improve vsn-2 ****
    //
    // Prev version :
    //   when insert key/doc, clone (key, document) for 
    //   send to reporter channel EVEN if not EXIST ANY subscriber
    // 
    // New version : ( ``` off_reporter() ``` )
    //   if reporter is off dont clone 
    //

    let s = Arc::new(Storage::<Pid, User>::open(ops)
                        .await
                        .unwrap()
                        .off_reporter());
    
    
    s1.insert("+98 9370156893".to_owned(), User::new("DanyalMh1")).await;
    s1.insert("+98 939.......".to_owned(), User::new("DanyalMh2")).await;
    s1.insert("+98 939.......".to_owned(), User::new("DanyalMh3")).await;
    


    // **** New Feature vsn-2 ****
    //
    //  Note:     
    //     Persistent Getter and Setter is complete safe, 
    //     can run Insert/Remove/Lookup on storage also 
    //     concurrently call (copy_memtable_to_database) and (load_memtable_from_database) in that 
    //
    //
    let cfg_string = "host=localhost user=postgres".to_string();
    let pers = Persistent::connect(DatabaseName::Postgres, cfg_string).await.unwrap();
    let h = Handler;

    h.init().await;

    // Copy table to postgres
    pers.copy_memtable_to_database(s1, &h).await;

    
    // Load table from postgres
    pers.load_memtable_from_database(s1, &h).await;
}


struct Handler;
#[async_trait]

impl Setter<Pid, User> for Handler {
    
    async fn init(&self, session: &DatabaseSession) {
        // Query: If not exist create Table 
    }
    
    async fn handle_setter(&self, session: &DatabaseSession, key: &Key, document: &Document) -> Result<(), Stop> {
        if let DatabaseSession::Postgres(client) = session {
            // Query: Insert to Table
            //
            // return Ok(())
            
        }
    } 
} 

impl Getter<Pid, User> for Handler {
    async fn handle_getter(&self, session: &DatabaseSession) -> Vec<(Key, Document)> {
        if let DatabaseSession::Postgres(client) = session {
            // Query: Select * from Table
            //
            // let list = fill_vec(...);
            //
            // return list
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