use darkbird::{Options, Schema, StorageType, VecStorage};
use serde_derive::{Deserialize, Serialize};

fn factory_option(name: &str) -> Options {
    Options::new(".", name, 1000, StorageType::RamCopies, true)
}

#[tokio::main]
async fn main() {
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
