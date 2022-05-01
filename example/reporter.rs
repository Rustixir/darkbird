

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

    
    let ops = Options::new(path, storage_name, total_page_size, StorageType::RamCopies);


    let s = Storage::<Pid, User>::open(ops).await.unwrap();


    // ----------------------------------------------------------------
    
    // create channel
    let (sx, mut rx) = mpsc::channel::<Event<Pid, User>>(100);

    // spawn a task for receive event from reporter
    tokio::spawn(async move {

        // await on recv
        let event = rx.recv().await.unwrap();

        // handle event
        match event {
            Event::Query(RQuery::Insert(_key, _doc)) => {
                unimplemented!()
            }
            Event::Query(RQuery::Remove(_key)) => {
                unimplemented!()
            }
            Event::Subscribed(_key) => {
                unimplemented!()
            }
        }
    });

    // subscribe to storage for recieve event from reporter
    let _ = s.subscribe(sx).await;
    
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

