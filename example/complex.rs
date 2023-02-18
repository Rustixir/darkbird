


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


    // ***************************************************************************
    // **                                                                       **
    // **   1. Storage is built for high concurrency usage                      **
    // **   2. dont need to Mutex / Rwlock                                      **
    // **   3. it is safe for shared between thread                             **
    // **                                                                       **
    // ***************************************************************************
    
    

    // create storage and initialize other service (reporter, disk_log)    
    //
    // when calling storage::open, it load whole storage from disk
    //
    let s = Arc::new(Storage::<Pid, User>::open(ops).await.unwrap());
    
        
        
    // spawn a task for inserting to storage
    let s1 = s.clone();    
    tokio::spawn(async move {
        let _ = s1.insert("98 000xxxxx1".to_owned(), User::new("DanyalMh")).await;
    });



    // spawn another task for inserting to storage
    let s2 = s.clone();    
    tokio::spawn(async move {
        let _ = s2.insert("98 000xxxxx1".to_owned(), User::new("DanyalMh")).await;
    });



    let s3 = s.clone();
    tokio::spawn(async move {
        // iter over storage
        s3.iter().for_each(|r| {
            println!("==> {}-{:?}", r.key(), r.value());
        })
    });

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



