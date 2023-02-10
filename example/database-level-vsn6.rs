use std::time::Duration;

use darkbird::Schema;
use tokio::time::sleep;


#[tokio::main]
async fn main() {
    

    // RedisStore is not support durable mode 

    let db = Schema::new()
            .with_redisstore::<i32, AuthData>("AuthData").await.unwrap()
            .build();


    let key = 10;
    let auth = AuthData::new("x210xawsxwadwaas", "0xaxacea213231172asx");
    let expire = Duration::from_secs(1);


    // set to storage
    db.set::<i32, AuthData>(key, auth.clone(), Some(expire.clone())).unwrap();


    // set if Not eXist
    let result = db.set_nx::<i32, AuthData>(key, auth, Some(expire)).unwrap();
    if !result {
        println!("Key exist.")
    }


    sleep(Duration::from_secs(2)).await;


    // get from storage
    if let Some(arc_doc) = db.get::<i32, AuthData>(&key).unwrap() {
        println!("==> We expected it to be removed. {}-{}", arc_doc.refresh_token, arc_doc.access_token);
    } else {
        println!("==> removed !!")
    }


    db.del::<i32, AuthData>(&key).unwrap()

}


#[derive(Clone)]
struct AuthData {
    refresh_token: String,
    access_token: String
}
impl AuthData {
    pub fn new(rt: &str, at: &str) -> Self {
        AuthData { 
            refresh_token: rt.to_owned(), 
            access_token: at.to_owned() 
        }
    }
}
