use serde::{Serialize, Deserialize};



pub type VectorId = String;




#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Vector(pub Vec<f32>);

impl Vector {
    /// Calculates the Euclidean distance between two vectors.
    ///
    /// The Euclidean distance is the square root of the sum of the squared differences
    /// between corresponding elements of the two vectors. The vectors must have the same
    /// number of dimensions.
    ///
    /// # Arguments
    ///
    /// * `v` - vector.
    ///
    /// # Returns
    ///
    /// The Euclidean distance between the two vectors as a floating-point value.
    ///
    /// # Example
    ///
    /// ```
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
    /// // Define two vectors
    /// let vector1 = vec![1.0, 2.0, 3.0];
    /// let vector2 = vec![4.0, 5.0, 6.0];
    ///
    /// // Calculate the Euclidean distance between the vectors
    /// let distance = vector1.euclidean_distance(&vector2);
    /// assert_eq!(distance, (27.0 as f32).sqrt());
    /// ```
    pub fn euclidean_distance(&self, v: &Vector) -> f32 {
        self.0.iter()
            .zip(v.0.iter())
            .map(|(x, y)| (x - y).powi(2))
            .sum::<f32>()
            .sqrt()
    }


    /// Calculates the cosine similarity between two vectors.
    ///
    /// The cosine similarity is a measure of similarity between two vectors in a
    /// multi-dimensional space. It is defined as the cosine of the angle between
    /// the vectors. The vectors must have the same number of dimensions.
    ///
    /// # Arguments
    ///
    /// * `v1` - vector.
    ///
    /// # Returns
    ///
    /// An `Option` containing the cosine similarity between the two vectors as a floating-point value.
    /// Returns `None` if the vectors have different dimensions.
    ///
    /// # Example
    ///
    /// ```
    ///
    /// use darkbird::Vector;
    /// 
    /// // Define two vectors
    /// let vector1 = Vector(vec![1.0, 2.0, 3.0]);
    /// let vector2 = Vector(vec![4.0, 5.0, 6.0]);
    ///
    /// // Calculate the cosine similarity between the vectors
    /// let similarity = vector1.cosine_similarity(&vector2).unwrap();
    /// assert_eq!(similarity, 0.9746318461970762);
    /// ```
    pub fn cosine_similarity(&self, v: &Vector) -> Option<f32> {
        if self.0.len() != v.0.len() {
            return None;
        }
        let dot_product = self.0.iter().zip(v.0.iter()).map(|(x, y)| x * y).sum::<f32>();
        let magnitude_v1 = (self.0.iter().map(|x| x.powi(2)).sum::<f32>()).sqrt();
        let magnitude_v2 = (v.0.iter().map(|x| x.powi(2)).sum::<f32>()).sqrt();
        Some(dot_product / (magnitude_v1 * magnitude_v2))
    }

}