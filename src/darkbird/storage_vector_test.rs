use crate::{VecStorage, Options, Vector, Schema, StorageType};

fn factory_option(name: &str) -> Options {
    Options::new(".", name, 1000, StorageType::RamCopies, true)
}


#[tokio::test]
async fn vector() {
    let path = ".";
    let storage_name = "vectors";
    let total_page_size = 500;
    let stype = crate::StorageType::RamCopies;
    let off_reporter = true;

    let ops = Options::new(path, storage_name, total_page_size, stype, off_reporter);
    let vstorage = VecStorage::open(ops).await.unwrap();


    let vid = String::from("vector_id");
    let vec =vec![1.2, 1.3, 0.5, 0.4];
    

    vstorage.insert(vid.clone(), vec.clone()).await.unwrap();


    let (vid2, vec2) = vstorage.lookup(&vid).unwrap();

    assert_eq!(vid, vid2);

    assert_eq!(Vector(vec).cosine_similarity(&vec2).unwrap(), 0f32)
}




#[tokio::test]
async fn k_nearest_neighbors() {
    let db = Schema::new()
        .with_vecstore(factory_option("LLM"))
        .await
        .unwrap()
        .build();

    // insert with vector_id
    let _ = db.vec_insert("vector1".to_string(), vec![1.0, 2.0, 3.0]).await.unwrap();
    let _ = db.vec_insert("vector2".to_string(), vec![4.0, 5.0, 6.0]).await.unwrap();
    let _ = db.vec_insert("vector3".to_string(), vec![7.0, 8.0, 9.0]).await.unwrap();


    // also can insert with uuid
    let _ = db.vec_insert_with_uuid(vec![10.0, 0.0, 10.0]).await.unwrap();

    
    // query
    let query_vector = Vector(vec![2.0, 3.0, 4.0]);

    // search
    let nearest_neighbors = db.vec_k_nearest_neighbors(&query_vector, 2).unwrap();

    // check
    assert_eq!(
        nearest_neighbors,
        vec![
            ("vector1".to_string(), Vector(vec![1.0, 2.0, 3.0])),
            ("vector2".to_string(), Vector(vec![4.0, 5.0, 6.0]))
        ]
    );
}
