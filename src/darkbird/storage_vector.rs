use tokio::sync::mpsc::Sender;

use dashmap::{iter::Iter, DashMap};
use uuid::Uuid;


use super::{
    wal::disk_log::{DiskLog, Session},
    router::{self, Router},
    Options, StatusResult, StorageType, vector::{VectorId, Vector},
};

use crate::{darkbird::SessionResult, RQuery, Event};



pub struct VecStorage {
    // DashMap
    vcache: DashMap<VectorId, Vector>,

    // Wal session
    wal_session: Session,

    // Reporter session
    reporter_session: router::Session<Event<VectorId, Vector>>,

    off_reporter: bool,

    off_disk: bool
}

impl VecStorage
{
    pub async fn open<'a>(ops: Options<'a>) -> Result<Self, String> {
        
        match DiskLog::open(ops.path, ops.storage_name, ops.total_page_size) {
            Err(e) => return Err(e.to_string()),
            Ok(disklog) => {
                // Run DiskLog
                let off_disk = if let StorageType::RamCopies = ops.stype { true } else { false };

                // Run Reporter
                let reporter = Router::<Event<VectorId, Vector>>::new(vec![]).unwrap().run_service();

                // Run disk_log
                let wal_session = disklog.run_service();


                // Create VecStorage
                let mut st = VecStorage {
                    vcache: DashMap::new(),
                    wal_session: wal_session,
                    reporter_session: reporter,
                    off_reporter: ops.off_reporter,
                    off_disk: true
                };


                // load from disk
                if let Err(x) = st.loader().await {
                    if x != "End" {
                        return Err(x);
                    } 
                }


                // because we want loader dont write to disk_log
                st.off_disk = off_disk;

                return Ok(st);
            }
        }

    }

    /// subscribe to Reporter
    #[inline]
    pub async fn subscribe(&self, sender: Sender<Event<VectorId, Vector>>) -> Result<(), SessionResult> {
        if self.off_reporter {
            return Err(SessionResult::Err(StatusResult::ReporterIsOff));
        }

        // Send to Reporter
        let _ = self
            .reporter_session
            .dispatch(Event::Subscribed(sender.clone()))
            .await;

        self.reporter_session.register(sender).await
    }

    /// insert to storage and persist to disk
    #[inline]
    pub async fn insert(&self, vid: VectorId, vec: Vec<f32>) -> Result<(), SessionResult> {
        let v = Vector(vec);
        if !self.off_disk || !self.off_reporter {
            let query = RQuery::Insert(vid.clone(), v.clone());

            if !self.off_disk {
                if let Err(e) = self.wal_session.log(bincode::serialize(&query).unwrap()).await {
                    return Err(e);
                }
            }

            if !self.off_reporter {
                let _ = self.reporter_session.dispatch(Event::Query(query)).await;
            }

        }


        // Insert to memory
        self.vcache.insert(vid, v);


        Ok(())


    }


    /// insert to storage and persist to disk
    #[inline]
    pub async fn insert_with_uuid(&self, vec: Vec<f32>) -> Result<(), SessionResult> {
        let vid = Uuid::new_v4().to_string();
        return self.insert(vid, vec).await
    }


    /// remove from storage and persist to disk
    #[inline]
    pub async fn remove(&self, vid: VectorId) -> Result<(), SessionResult> {
        match self.vcache.get(&vid) {
            Some(_) => {

                if !self.off_disk || !self.off_reporter {
                    let query = RQuery::<VectorId, Vector>::Remove(vid.clone());
        
                    if !self.off_disk {
                        if let Err(e) = self.wal_session.log(bincode::serialize(&query).unwrap()).await {
                            return Err(e);
                        }
                    }
        
                    if !self.off_reporter {
                        let _ = self.reporter_session.dispatch(Event::Query(query)).await;
                    }
                    
                }

                self.vcache.remove(&vid);

            }
            None => return Ok(()),
        }

        Ok(())
    }

    /// gets documents  
    #[inline]
    pub fn gets(&self, list: Vec<VectorId>) -> Vec<(VectorId, Vector)> {
        let mut result = Vec::with_capacity(list.len());

        list.iter().for_each(|vid| {
            if let Some(r) = self.vcache.get(vid) {
                result.push((r.key().clone(), r.value().clone()));
            }
        });

        result
    }

    /// lookup by vector_id
    #[inline]
    pub fn lookup(&self, vid: &VectorId) -> Option<(VectorId, Vector)> {
        match self.vcache.get(vid) {
            Some(v) => {
                return Some((v.key().clone(), Vector(v.0.clone())))
            }
            None => return None,
        }
    }

    /// Finds the k-nearest neighbors to a given query vector in the storage.
    ///
    /// The k-nearest neighbors are determined based on the Euclidean distance between
    /// the query vector and the vectors stored in the storage. The function returns
    /// a vector of tuples, where each tuple contains the key (ID) and a reference to
    /// one of the k-nearest neighbor vectors.
    ///
    /// # Arguments
    ///
    /// * `query` - The query vector for which the k-nearest neighbors are to be found.
    /// * `k` - The number of nearest neighbors to retrieve.
    ///
    /// # Returns
    ///
    /// A vector of tuples containing the keys (IDs) and references to the k-nearest
    /// neighbor vectors. If there are fewer than k vectors in the storage, the function
    /// returns all available vectors.
    ///
    /// # Example
    ///
    /// ```
    /// use darkbird::{Options, storage_vector::{self, VecStorage}};
    ///
    /// // Create a new storage instance
    /// 
    /// let path = ".";
    /// let storage_name = "vectors";
    /// let total_page_size = 500;
    /// let stype = crate::StorageType::RamCopies;
    /// let off_reporter = true;
    /// 
    /// let ops = Options::new(path, storage_name, total_page_size, stype, off_reporter);
    /// let db = VecStorage::open(ops).await.unwrap();
    ///
    /// // Insert vectors into the storage
    /// db.insert_with_key("vector1".to_string(), vec![1.0, 2.0, 3.0]);
    /// db.insert_with_key("vector2".to_string(), vec![4.0, 5.0, 6.0]);
    /// db.insert_with_key("vector3".to_string(), vec![7.0, 8.0, 9.0]);
    ///
    /// // Define a query vector
    /// let query_vector = vec![2.0, 3.0, 4.0];
    ///
    /// // Find the 2 nearest neighbors to the query vector
    /// let nearest_neighbors = db.k_nearest_neighbors(&query_vector, 2);
    /// assert_eq!(nearest_neighbors, vec![
    ///     ("vector1".to_string(), vec![1.0, 2.0, 3.0]),
    ///     ("vector2".to_string(), vec![4.0, 5.0, 6.0])
    /// ]);
    /// ```
    #[inline]
    pub fn k_nearest_neighbors(&self, query: &Vector, k: usize) -> Vec<(VectorId, Vector)> {
        let mut neighbors = self
            .vcache
            .iter()
            .map(|rf| (rf.key().clone(), query.euclidean_distance(rf.value())))
            .collect::<Vec<_>>();
        neighbors.sort_by(|(_, dist1), (_, dist2)| dist1.partial_cmp(dist2).unwrap());
        neighbors
            .into_iter()
            .take(k)
            .map(|(id, _)| (id.clone(), self.vcache.get(&id).unwrap().value().clone()))
            .collect()
    }
    
    /// Performs element-wise addition of two vectors stored in the darkbird storage.
    ///
    /// The vectors are identified by their keys (IDs). The function returns the result
    /// of the addition as a new vector. The vectors must have the same number of dimensions.
    ///
    /// # Arguments
    ///
    /// * `key1` - The key (ID) of the first vector to be added.
    /// * `key2` - The key (ID) of the second vector to be added.
    ///
    /// # Returns
    ///
    /// An `Option` containing the result of the vector addition as a new vector.
    /// Returns `None` if either of the keys is not found in the storage, or if the
    /// vectors have different dimensions.
    ///
    /// # Example
    ///
    /// ```
    /// use darkbird::{Options, storage_vector::{self, VecStorage}};
    ///
    /// // Create a new storage instance
    /// 
    /// let path = ".";
    /// let storage_name = "vectors";
    /// let total_page_size = 500;
    /// let stype = crate::StorageType::RamCopies;
    /// let off_reporter = true;
    /// 
    /// let ops = Options::new(path, storage_name, total_page_size, stype, off_reporter);
    /// let db = VecStorage::open(ops).await.unwrap();
    ///
    /// // Insert vectors into the storage
    /// db.insert_with_key("vector1".to_string(), vec![1.0, 2.0, 3.0]);
    /// db.insert_with_key("vector2".to_string(), vec![4.0, 5.0, 6.0]);
    ///
    /// // Perform vector addition
    /// let result = db.vector_addition("vector1", "vector2");
    /// assert_eq!(result, Some(vec![5.0, 7.0, 9.0]));
    /// ```
    pub fn vector_addition(&self, vec_id1: &str, vec_id2: &str) -> Option<Vector> {
        let v1 = self.vcache.get(vec_id1)?;
        let v2 = self.vcache.get(vec_id2)?;
        if v1.0.len() != v2.0.len() {
            return None;
        }
        
        let v = v1.0.iter().zip(v2.0.iter()).map(|(x, y)| x + y).collect::<Vec<f32>>();
        return Some(Vector(v))
    }


        /// Performs element-wise subtraction of two vectors stored in the storage.
    ///
    /// The vectors are identified by their keys (IDs). The function returns the result
    /// of the subtraction as a new vector. The vectors must have the same number of dimensions.
    ///
    /// # Arguments
    ///
    /// * `key1` - The key (ID) of the minuend vector.
    /// * `key2` - The key (ID) of the subtrahend vector.
    ///
    /// # Returns
    ///
    /// An `Option` containing the result of the vector subtraction as a new vector.
    /// Returns `None` if either of the keys is not found in the storage, or if the
    /// vectors have different dimensions.
    ///
    /// # Example
    ///
    /// ```
    /// use darkbird::{Options, storage_vector::{self, VecStorage}};
    ///
    /// // Create a new storage instance
    /// 
    /// let path = ".";
    /// let storage_name = "vectors";
    /// let total_page_size = 500;
    /// let stype = crate::StorageType::RamCopies;
    /// let off_reporter = true;
    /// 
    /// let ops = Options::new(path, storage_name, total_page_size, stype, off_reporter);
    /// let db = VecStorage::open(ops).await.unwrap();
    ///
    /// // Insert vectors into the storage
    /// db.insert_with_key("vector1".to_string(), vec![1.0, 2.0, 3.0]);
    /// db.insert_with_key("vector2".to_string(), vec![4.0, 5.0, 6.0]);
    ///
    /// // Perform vector subtraction
    /// let result = db.vector_subtraction("vector1", "vector2");
    /// assert_eq!(result, Some(vec![-3.0, -3.0, -3.0]));
    /// ```
    pub fn vector_subtraction(&self, key1: &str, key2: &str) -> Option<Vector> {
        let v1 = self.vcache.get(key1)?;
        let v2 = self.vcache.get(key2)?;
        if v1.0.len() != v2.0.len() {
            return None;
        }

        let v = v1.0.iter().zip(v2.0.iter()).map(|(x, y)| x - y).collect::<Vec<f32>>();
        return Some(Vector(v))
    }

    /// Performs scalar multiplication of a vector stored in the storage.
    ///
    /// The vector is identified by its key (ID). The function returns the result
    /// of the scalar multiplication as a new vector.
    ///
    /// # Arguments
    ///
    /// * `key` - The key (ID) of the vector to be scaled.
    /// * `scalar` - The scalar value by which the vector is to be multiplied.
    ///
    /// # Returns
    ///
    /// An `Option` containing the result of the vector scaling as a new vector.
    /// Returns `None` if the key is not found in the storage.
    ///
    /// # Example
    ///
    /// ```
    /// use darkbird::{Options, storage_vector::{self, VecStorage}};
    ///
    /// // Create a new storage instance
    /// 
    /// let path = ".";
    /// let storage_name = "vectors";
    /// let total_page_size = 500;
    /// let stype = crate::StorageType::RamCopies;
    /// let off_reporter = true;
    /// 
    /// let ops = Options::new(path, storage_name, total_page_size, stype, off_reporter);
    /// let db = VecStorage::open(ops).await.unwrap();
    ///
    /// // Insert a vector into the storage
    /// db.insert_with_key("vector1".to_string(), vec![1.0, 2.0, 3.0]);
    ///
    /// // Perform vector scaling
    /// let result = db.vector_scaling("vector1", 2.0);
    /// assert_eq!(result, Some(vec![2.0, 4.0, 6.0]));
    /// ```
    pub fn vector_scaling(&self, key: &str, scalar: f32) -> Option<Vector> {
        let v = self.vcache.get(key)?;
        let v = v.0.iter().map(|x| x * scalar).collect();
        return Some(Vector(v))
    }



    /// return Iter (Safe for mutation)
    #[inline]
    pub fn iter(&self) -> Iter<'_, VectorId, Vector> {
        self.vcache.iter()
    }


    

    /// load storage from disk
    #[inline]
    async fn loader(&self) -> Result<(), String> {
        // when storage just open with Disc Copies option it call loader, else it don't call
        let wal = &self.wal_session;

        let mut page_index = 1;

        loop {
            // Get Page
            let mut logfile = match wal.get_page(page_index).await {
                Ok(lf) => lf,
                Err(sess_res) => {
                    if let SessionResult::Err(e) = sess_res {
                        return Err(e.to_string())
                    }

                    return Err("disk_log closed".to_string())
                }
            };

            page_index += 1;

            // Must Call Recover if return Err, remove unwrap()
            let iter = match logfile.iter(..) {
                Ok(iter) => iter,
                Err(e) => {
                    eprintln!("==> {:?}", e);
                    return Err(e.to_string());
                }
            };

            for qline in iter {
                let bytes = match qline {
                    Ok(ql)  => ql,
                    Err(e) => return Err(e.to_string()),
                };

                let query: RQuery<VectorId, Vector> = match bincode::deserialize(&bytes) {
                    Ok(rq) => rq,
                    Err(e) => {
                        return Err(e.to_string());
                    }
                };

                match query {
                    RQuery::Insert(vid, v) => {                        
                        let _ = self.insert(vid, v.0).await;
                    }
                    RQuery::Remove(vid) => {
                        let _ = self.remove(vid).await;
                    }
                }
            }
        }
    }


}
