use rayon::prelude::*;
use std::collections::{BinaryHeap, HashMap};
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use crate::similarity::{get_cache_attr, get_distance_fn, normalize, ScoreIndex};
use crate::model::{CacheDB, SimilarityResult, Collection, Embedding, Distance, Error};
use log::{debug, error, info, trace, warn};
use std::sync::Once;

static INIT: Once = Once::new();

fn setup_logger() -> Result<(), fern::InitError> {
    INIT.call_once(|| {
        let _ = fern::Dispatch::new()
            .format(|out, message, record| {
                out.finish(format_args!(
                    "{} [{}] {}",
                    chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                    record.level(),
                    message
                ))
            })
            .level(log::LevelFilter::Info)
            .chain(std::io::stdout())
            .chain(fern::log_file("output.log").unwrap())
            .apply();
    });
    Ok(())
}


// Define a function to hash a HashMap<String, String>.
// A custom hash function, you ensure that the hash value is based solely on the content of the HashMap
pub fn hash_map_id(id: &HashMap<String, String>) -> u64 {
    let mut hasher = DefaultHasher::new();
    for (key, value) in id {
        key.hash(&mut hasher);
        value.hash(&mut hasher);
    }
    hasher.finish()
}

/// A collection that stores embeddings and handles similarity calculations.
impl Collection {
    /// Calculate similarity results for a given query and number of results (k).
    ///
    /// # Arguments
    ///
    /// * `query`: The query vector for which to calculate similarity.
    /// * `k`: The number of top similar results to return.
    ///
    /// # Returns
    ///
    /// A vector of similarity results, sorted by their similarity scores.
    pub fn get_similarity(&self, query: &[f32], k: usize) -> Vec<SimilarityResult> {

        debug!("Starting similarity computation with query vector of length {} and top k = {}", query.len(), k);

        // Prepare cache attributes and distance function based on collection's distance metric.
        let memo_attr = get_cache_attr(self.distance, query);
        let distance_fn = get_distance_fn(self.distance);

        debug!("Using distance function: {:?}", self.distance);
        debug!("Memo attributes for distance function: {:?}", memo_attr);

        // Calculate similarity scores for each embedding in parallel.
        let scores = self.embeddings.par_iter()
            .enumerate()
            .map(|(index, embedding)| {
                let score = distance_fn(&embedding.vector, query, memo_attr);
                ScoreIndex { score, index }
            })
            .collect::<Vec<_>>();
        debug!("Calculated {} similarity scores", scores.len());
        // Use a binary heap to efficiently find the top k similarity results.
        let mut heap = BinaryHeap::new();
        for score_index in scores {
            // Only keep top k results in the heap.
            if heap.len() < k || score_index < *heap.peek().unwrap() {
                heap.push(score_index);
                if heap.len() > k {
                    heap.pop();
                }
            }
        }
        debug!("Top k heap size: {}", heap.len());

        // Convert the heap into a sorted vector and map each score to a SimilarityResult.
        let result: Vec<SimilarityResult> = heap.into_sorted_vec()
            .into_iter()
            .map(|ScoreIndex { score, index }| SimilarityResult {
                score,
                embedding: self.embeddings[index].clone(),
            })
            .collect();
        info!("Similarity computed successfully'{}' ", format!("{:?}", result));
        result
    }
}

/// Database management functionality for collections of embeddings.
impl CacheDB {
    /// Initialize a new CacheDB instance.
    pub fn new() -> Self {
        Self {
            collections: HashMap::new(),
        }
    }
    /// Create a new collection in the database.
    ///
    /// # Arguments
    ///
    /// * `name`: The name of the collection to create.
    /// * `dimension`: The dimension of the embeddings in the collection.
    /// * `distance`: The distance metric to use for similarity calculations.
    ///
    /// # Returns
    ///
    /// A result containing the new collection or an error if a collection with the same name already exists.
    pub fn create_collection(
        &mut self,
        name: String,
        dimension: usize,
        distance: Distance,
    ) -> Result<Collection, Error> {

        if let Err(e) = setup_logger() {
            error!("Logger setup failed: {:?}", e);
            return Err(Error::LoggerInitializationError);
        }

        // Check if a collection with the same name already exists.
        if self.collections.contains_key(&name) {
            error!("Collection: '{}', already exists", name);
            return Err(Error::UniqueViolation);
        }

        // Create a new collection and add it to the database.
        let collection = Collection {
            dimension,
            distance,
            embeddings: Vec::new(),
        };
        self.collections.insert(name.clone(), collection.clone());

        info!("Created new collection with name: '{}'", name);
        Ok(collection)
    }

    /// Delete a collection from the database.
    ///
    /// # Arguments
    ///
    /// * `name`: The name of the collection to delete.
    ///
    /// # Returns
    ///
    /// A result indicating success or an error if the collection was not found.
    pub fn delete_collection(&mut self, name: &str) -> Result<(), Error> {

        if let Err(e) = setup_logger() {
            error!("Logger setup failed: {:?}", e);
            return Err(Error::LoggerInitializationError);
        }

        // Check if the collection exists before attempting to delete it.
        if !self.collections.contains_key(name) {
            error!("Collection name: '{}', does not exist", name);
            return Err(Error::NotFound);
        }

        // Remove the collection from the database.
        self.collections.remove(name);

        info!("Deleted collection with name: '{}'", name);
        Ok(())
    }

    /// Insert a new embedding into a specified collection.
    ///
    /// # Arguments
    ///
    /// * `collection_name`: The name of the collection to insert the embedding into.
    /// * `embedding`: The embedding to insert.
    ///
    /// # Returns
    ///
    /// A result indicating success or an error if the collection was not found, the embedding is a duplicate, or the embedding dimension does not match the collection.
    pub fn insert_into_collection(
        &mut self,
        collection_name: &str,
        mut embedding: Embedding,
    ) -> Result<(), Error> {

        if let Err(e) = setup_logger() {
            error!("Logger setup failed: {:?}", e);
            return Err(Error::LoggerInitializationError);
        }

        // Get the collection to insert the embedding into.
        let collection = self.collections
            .get_mut(collection_name)
            .ok_or(Error::NotFound)?;
        

        // Create a HashSet to track unique hashed IDs.
        let mut unique_ids: HashSet<u64> = collection.embeddings
            .iter()
            .map(|e| hash_map_id(&e.id))
            .collect();

        // Check for duplicate embeddings by hashed ID.
        if !unique_ids.insert(hash_map_id(&embedding.id)) {
            error!("Embedding with ID '{}' already exists in collection '{}'", format!("{:?}", embedding.id), collection_name);
            return Err(Error::EmbeddingUniqueViolation);
        }

        // Check if the embedding's dimension matches the collection's dimension.
        if embedding.vector.len() != collection.dimension {
            error!(
                "Dimension mismatch: embedding vector length is '{}' but collection '{}' expects dimension '{}'",
                embedding.vector.len(),
                collection_name,
                collection.dimension
            );
            return Err(Error::DimensionMismatch);
        }

        // Normalize the embedding vector if using cosine distance for more efficient calculations.
        if collection.distance == Distance::Cosine {
            embedding.vector = normalize(&embedding.vector);
        }

        // Add the embedding to the collection.
        collection.embeddings.push(embedding);

        info!("Embedding successfully inserted into collection '{}'", collection_name);
        Ok(())
    }

    /// Update a collection with new embeddings.
    ///
    /// # Arguments
    ///
    /// * `collection_name`: The name of the collection to update.
    /// * `new_embeddings`: A vector of new embeddings to add to the collection.
    ///
    /// # Returns
    ///
    /// A result indicating success or an error if the collection was not found, there are duplicate embeddings, or the embedding dimensions do not match the collection's dimension.
    pub fn update_collection(
        &mut self,
        collection_name: &str,
        new_embeddings: Vec<Embedding>,
    ) -> Result<(), Error> {

        if let Err(e) = setup_logger() {
            error!("Logger setup failed: {:?}", e);
            return Err(Error::LoggerInitializationError);
        }

        // Get the collection to update.
        let collection = self.collections
            .get_mut(collection_name)
            .ok_or(Error::NotFound)?;

        // Iterate through each new embedding.
        for mut embedding in new_embeddings {
            // Create a HashSet to track unique hashed IDs.
            let mut unique_ids: HashSet<u64> = collection.embeddings
            .iter()
            .map(|e| hash_map_id(&e.id))
            .collect();

            // Check for duplicate embeddings by hashed ID.
            if !unique_ids.insert(hash_map_id(&embedding.id)) {
                error!("Embedding with ID '{}' already exists in collection '{}'", format!("{:?}", embedding.id), collection_name);
                return Err(Error::UniqueViolation);
            }

            // Check if the embedding's dimension matches the collection's dimension.
            if embedding.vector.len() != collection.dimension {
                error!(
                    "Dimension mismatch: embedding vector length is '{}' but collection '{}' expects dimension '{}'",
                    embedding.vector.len(),
                    collection_name,
                    collection.dimension
                );
                return Err(Error::DimensionMismatch);
            }

            // Normalize the vector if using cosine distance for efficient calculations.
            if collection.distance == Distance::Cosine {
                embedding.vector = normalize(&embedding.vector);
            }

            // Add the embedding to the collection.
            collection.embeddings.push(embedding);
        }

        info!("Embedding successfully updated to collection '{}'", collection_name);
        Ok(())
    }

    /// Retrieve a collection from the database.
    ///
    /// # Arguments
    ///
    /// * `collection_name`: The name of the collection to retrieve.
    ///
    /// # Returns
    ///
    /// An optional reference to the collection if found.
    pub fn get_collection(&self, collection_name: &str) -> Option<&Collection> {
        if let Err(e) = setup_logger() {
            error!("Logger setup failed: {:?}", e);
        }
    
        match self.collections.get(collection_name) {
            Some(collection) => {
                info!("Collection '{}' found", collection_name);
                Some(collection)
            },
            None => {
                error!("Collection '{}' not found", collection_name);
                None
            }
        }
    }

    /// Retrieve embeddings from a collection in the database.
    ///
    /// # Arguments
    ///
    /// * `collection_name`: The name of the collection to retrieve.
    ///
    /// # Returns
    ///
    /// An optional reference to the embeddings if found.
    pub fn get_embeddings(&self, collection_name: &str) -> Option<Vec<Embedding>> {
        if let Err(e) = setup_logger() {
            error!("Logger setup failed: {:?}", e);
        }
    
        match self.collections.get(collection_name) {
            Some(collection) => {
                info!("Successfully retrieved embeddings for collection '{}'", collection_name);
                Some(collection.embeddings.clone())
            },
            None => {
                error!("Collection '{}' not found", collection_name);
                None
            }
        }
    }  
}







#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_collection_success_eucledean() {
        let mut db = CacheDB::new();
        let result = db.create_collection("test_collection".to_string(), 100, Distance::Euclidean);

        assert!(result.is_ok());
        let collection = result.unwrap();
        assert_eq!(collection.dimension, 100);
        assert_eq!(collection.distance, Distance::Euclidean);
        assert!(db.collections.contains_key("test_collection"));
    }

    #[test]
    fn test_create_collection_success_cosine() {
        let mut db = CacheDB::new();
        let result = db.create_collection("test_collection".to_string(), 100, Distance::Cosine);

        assert!(result.is_ok());
        let collection = result.unwrap();
        assert_eq!(collection.dimension, 100);
        assert_eq!(collection.distance, Distance::Cosine);
        assert!(db.collections.contains_key("test_collection"));
    }

    #[test]
    fn test_create_collection_success_dot_product() {
        let mut db = CacheDB::new();
        let result = db.create_collection("test_collection".to_string(), 100, Distance::DotProduct);

        assert!(result.is_ok());
        let collection = result.unwrap();
        assert_eq!(collection.dimension, 100);
        assert_eq!(collection.distance, Distance::DotProduct);
        assert!(db.collections.contains_key("test_collection"));
    }


    #[test]
    fn test_create_collection_already_exists() {
        let mut db = CacheDB::new();
        db.create_collection("test_collection".to_string(), 100, Distance::Euclidean).unwrap();

        let result = db.create_collection("test_collection".to_string(), 200, Distance::Cosine);
        assert!(result.is_err());
    }

    #[test]
    fn test_insert_into_collection_success() {
        let mut db = CacheDB::new();
        let collection = Collection {
            dimension: 3,
            distance: Distance::Euclidean,
            embeddings: Vec::new(),
        };
        db.collections.insert("test_collection".to_string(), collection);
        let mut metadata = HashMap::new();
        metadata.insert("page".to_string(), "1".to_string());
        metadata.insert("text".to_string(), "This is a test metadata text".to_string());

        let mut id = HashMap::new();
        id.insert("unique_id".to_string(), "1".to_string());
        
        let embedding = Embedding {
            id: id,
            vector: vec![1.0, 2.0, 3.0],
            metadata: Some(metadata)
        };

        let result = db.insert_into_collection("test_collection", embedding.clone());
        assert!(result.is_ok());

        // Check if the embedding is inserted into the collection
        let collection = db.collections.get("test_collection").unwrap();
        assert_eq!(collection.embeddings.len(), 1);
        assert_eq!(collection.embeddings[0], embedding);
    }


    #[test]
    fn test_update_collection_success() {
        let mut db = CacheDB::new();

        let mut metadata = HashMap::new();
        metadata.insert("page".to_string(), "1".to_string());
        metadata.insert("text".to_string(), "This is a test metadata text".to_string());

        let mut id = HashMap::new();
        id.insert("unique_id".to_string(), "0".to_string());

        let collection = Collection {
            dimension: 3,
            distance: Distance::Euclidean,
            embeddings: vec![Embedding {
                id: id,
                vector: vec![1.0, 2.0, 3.0],
                metadata: Some(metadata.clone())
            }],
        };

        db.collections.insert("test_collection".to_string(), collection);

        let mut id_1 = HashMap::new();
        id_1.insert("unique_id".to_string(), "1".to_string());
        let mut id_2 = HashMap::new();
        id_2.insert("unique_id".to_string(), "2".to_string());

        let new_embeddings = vec![
            Embedding {
                id: id_1, // Duplicate ID
                vector: vec![4.0, 5.0, 6.0],
                metadata: Some(metadata.clone())
            },
            Embedding {
                id: id_2,
                vector: vec![7.0, 8.0, 9.0],
                metadata: Some(metadata.clone())
            },
        ];

        let result = db.update_collection("test_collection", new_embeddings.clone());
        assert!(result.is_ok());

        // Check if the new embeddings are added to the collection
        let collection = db.collections.get("test_collection").unwrap();
        assert_eq!(collection.embeddings.len(), 3);
        assert_eq!(collection.embeddings[1..], new_embeddings[..]);
    }

    #[test]
    fn test_update_collection_duplicate_embedding() {
        let mut db = CacheDB::new();
        let mut metadata = HashMap::new();
        metadata.insert("page".to_string(), "1".to_string());
        metadata.insert("text".to_string(), "This is a test metadata text".to_string());

        let mut id = HashMap::new();
        id.insert("unique_id".to_string(), "0".to_string());

        let collection = Collection {
            dimension: 3,
            distance: Distance::Euclidean,
            embeddings: vec![Embedding {
                id: id.clone(),
                vector: vec![1.0, 2.0, 3.0],
                metadata: Some(metadata.clone())
            }],
        };
        db.collections.insert("test_collection".to_string(), collection);

        let mut id_1 = HashMap::new();
        id_1.insert("unique_id".to_string(), "1".to_string());
        let mut id_2 = HashMap::new();
        id_2.insert("unique_id".to_string(), "2".to_string());

        let new_embeddings = vec![
            Embedding {
                id: id, // Duplicate ID
                vector: vec![4.0, 5.0, 6.0],
                metadata: Some(metadata.clone())
            },
            Embedding {
                id: id_2,
                vector: vec![7.0, 8.0, 9.0],
                metadata: Some(metadata.clone())
            },
        ];

        let result = db.update_collection("test_collection", new_embeddings);
        assert!(result.is_err());
        assert_eq!(result.err(), Some(Error::UniqueViolation));
    }

    #[test]
    fn test_update_collection_dimension_mismatch() {
        let mut db = CacheDB::new();
        let collection = Collection {
            dimension: 3,
            distance: Distance::Euclidean,
            embeddings: Vec::new(),
        };
        db.collections.insert("test_collection".to_string(), collection);

        let mut metadata = HashMap::new();
        metadata.insert("page".to_string(), "1".to_string());
        metadata.insert("text".to_string(), "This is a test metadata text".to_string());

        let mut id = HashMap::new();
        id.insert("unique_id".to_string(), "0".to_string());

        let new_embeddings = vec![
            Embedding {
                id: id,
                vector: vec![1.0, 2.0], 
                metadata: Some(metadata)// Dimension mismatch
            },
        ];

        let result = db.update_collection("test_collection", new_embeddings);
        assert!(result.is_err());
        assert_eq!(result.err(), Some(Error::DimensionMismatch));
    }

    #[test]
    fn test_delete_collection_success() {
        let mut db = CacheDB::new();
        db.collections.insert("test_collection".to_string(), Collection {
            dimension: 3,
            distance: Distance::Euclidean,
            embeddings: Vec::new(),
        });

        let result = db.delete_collection("test_collection");
        assert!(result.is_ok());

        // Check if the collection is removed from the database
        assert!(!db.collections.contains_key("test_collection"));
    }

    #[test]
    fn test_delete_collection_not_found() {
        let mut db = CacheDB::new();

        let result = db.delete_collection("non_existent_collection");
        assert!(result.is_err());
        assert_eq!(result.err(), Some(Error::NotFound));
    }

    #[test]
    fn test_get_collection_success() {
        let mut db = CacheDB::new();
        let collection = Collection {
            dimension: 3,
            distance: Distance::Euclidean,
            embeddings: Vec::new(),
        };
        db.collections.insert("test_collection".to_string(), collection.clone());

        let result = db.get_collection("test_collection");
        assert!(result.is_some());

        // Check if the retrieved collection is the same as the original one
        assert_eq!(result.unwrap(), &collection);
    }

    #[test]
    fn test_get_collection_not_found() {
        let db = CacheDB::new();

        let result = db.get_collection("non_existent_collection");
        assert!(result.is_none());
    }


    #[test]
    fn test_get_embedding_success() {
        let mut db = CacheDB::new();

        let mut id = HashMap::new();
        id.insert("unique_id".to_string(), "0".to_string());

        let mut id_1 = HashMap::new();
        id_1.insert("unique_id".to_string(), "1".to_string());

        let mut id_2 = HashMap::new();
        id_2.insert("unique_id".to_string(), "2".to_string());

        let collection = Collection {
            dimension: 3,
            distance: Distance::Euclidean,
            embeddings: vec![
                Embedding { id: id, vector: vec![1.0, 1.0, 1.0], metadata: None },
                Embedding { id: id_1, vector: vec![2.0, 2.0, 2.0], metadata: None },
                Embedding { id: id_2, vector: vec![3.0, 3.0, 3.0], metadata: None },
            ],
        };
        db.collections.insert("test_collection".to_string(), collection.clone());
        let result = db.get_embeddings("test_collection");
        assert!(result.is_some());
        assert_eq!(result, Some(collection.embeddings));
    }

    #[test]
    fn test_get_embeddings_not_found() {
        let db = CacheDB::new();

        let result = db.get_embeddings("non_existent_collection");
        assert!(result.is_none());
    }



    #[test]
    fn test_get_similarity() {
        let mut id = HashMap::new();
        id.insert("unique_id".to_string(), "0".to_string());

        let mut id_1 = HashMap::new();
        id_1.insert("unique_id".to_string(), "1".to_string());

        let mut id_2 = HashMap::new();
        id_2.insert("unique_id".to_string(), "2".to_string());

        let collection = Collection {
            dimension: 3,
            distance: Distance::Euclidean,
            embeddings: vec![
                Embedding { id: id.clone(), vector: vec![1.0, 1.0, 1.0], metadata: None },
                Embedding { id: id_1.clone(), vector: vec![2.0, 2.0, 2.0], metadata: None },
                Embedding { id: id_2.clone(), vector: vec![3.0, 3.0, 3.0], metadata: None },
            ],
        };

        // Define a query vector
        let query = vec![0.0, 0.0, 0.0];

        // Define the expected similarity results
        let expected_results = vec![
            SimilarityResult { score: 0.0, embedding: Embedding { id: id_1, vector: vec![2.0, 2.0, 2.0], metadata: None } },
            SimilarityResult { score: 0.0, embedding: Embedding { id: id_2, vector: vec![3.0, 3.0, 3.0], metadata: None } },
            SimilarityResult { score: 0.0, embedding: Embedding { id: id, vector: vec![1.0, 1.0, 1.0], metadata: None } },
        ];

        // Call the get_similarity method
        let results = collection.get_similarity(&query, 3);

        // Assert that the results are as expected
        assert_eq!(results, expected_results);
    }

}
