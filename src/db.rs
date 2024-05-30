use rayon::prelude::*;
use std::collections::{BinaryHeap, HashMap};
use crate::similarity::{get_cache_attr, get_distance_fn, normalize, ScoreIndex};
use crate::model::{CacheDB, SimilarityResult, Collection, Embedding, Distance, Error};


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
        // Prepare cache attributes and distance function based on collection's distance metric.
        let memo_attr = get_cache_attr(self.distance, query);
        let distance_fn = get_distance_fn(self.distance);

        // Calculate similarity scores for each embedding in parallel.
        let scores = self.embeddings.par_iter()
            .enumerate()
            .map(|(index, embedding)| {
                let score = distance_fn(&embedding.vector, query, memo_attr);
                ScoreIndex { score, index }
            })
            .collect::<Vec<_>>();

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

        // Convert the heap into a sorted vector and map each score to a SimilarityResult.
        heap.into_sorted_vec()
            .into_iter()
            .map(|ScoreIndex { score, index }| SimilarityResult {
                score,
                embedding: self.embeddings[index].clone(),
            })
            .collect()
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
        // Check if a collection with the same name already exists.
        if self.collections.contains_key(&name) {
            return Err(Error::UniqueViolation);
        }

        // Create a new collection and add it to the database.
        let collection = Collection {
            dimension,
            distance,
            embeddings: Vec::new(),
        };
        self.collections.insert(name, collection.clone());

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
        // Check if the collection exists before attempting to delete it.
        if !self.collections.contains_key(name) {
            return Err(Error::NotFound);
        }

        // Remove the collection from the database.
        self.collections.remove(name);

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
        // Get the collection to insert the embedding into.
        let collection = self.collections
            .get_mut(collection_name)
            .ok_or(Error::NotFound)?;

        // Check for duplicate embeddings by ID.
        if collection.embeddings.iter().any(|e| e.id == embedding.id) {
            return Err(Error::UniqueViolation);
        }

        // Check if the embedding's dimension matches the collection's dimension.
        if embedding.vector.len() != collection.dimension {
            return Err(Error::DimensionMismatch);
        }

        // Normalize the embedding vector if using cosine distance for more efficient calculations.
        if collection.distance == Distance::Cosine {
            embedding.vector = normalize(&embedding.vector);
        }

        // Add the embedding to the collection.
        collection.embeddings.push(embedding);

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
        // Get the collection to update.
        let collection = self.collections
            .get_mut(collection_name)
            .ok_or(Error::NotFound)?;

        // Iterate through each new embedding.
        for mut embedding in new_embeddings {
            // Check for duplicate embeddings by ID.
            if collection.embeddings.iter().any(|e| e.id == embedding.id) {
                return Err(Error::UniqueViolation);
            }

            // Check if the embedding's dimension matches the collection's dimension.
            if embedding.vector.len() != collection.dimension {
                return Err(Error::DimensionMismatch);
            }

            // Normalize the vector if using cosine distance for efficient calculations.
            if collection.distance == Distance::Cosine {
                embedding.vector = normalize(&embedding.vector);
            }

            // Add the embedding to the collection.
            collection.embeddings.push(embedding);
        }

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
        self.collections.get(collection_name)
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
        self.collections.get(collection_name).map(|collection| collection.embeddings.clone())
    }    
}







#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::MetadataValue;

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
        metadata.insert("page".to_string(), MetadataValue::Str("1".to_string()));
        metadata.insert("text".to_string(), MetadataValue::Str("This is a test metadata text".to_string()));

        let embedding = Embedding {
            id: "1".to_string(),
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
        metadata.insert("page".to_string(), MetadataValue::Str("1".to_string()));
        metadata.insert("text".to_string(), MetadataValue::Str("This is a test metadata text".to_string()));
        let collection = Collection {
            dimension: 3,
            distance: Distance::Euclidean,
            embeddings: vec![Embedding {
                id: "1".to_string(),
                vector: vec![1.0, 2.0, 3.0],
                metadata: Some(metadata.clone())
            }],
        };
        db.collections.insert("test_collection".to_string(), collection);

        let new_embeddings = vec![
            Embedding {
                id: "2".to_string(),
                vector: vec![4.0, 5.0, 6.0],
                metadata: Some(metadata.clone()),
            },
            Embedding {
                id: "3".to_string(),
                vector: vec![7.0, 8.0, 9.0],
                metadata: Some(metadata.clone()),
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
        metadata.insert("page".to_string(), MetadataValue::Str("1".to_string()));
        metadata.insert("text".to_string(), MetadataValue::Str("This is a test metadata text".to_string()));
        let collection = Collection {
            dimension: 3,
            distance: Distance::Euclidean,
            embeddings: vec![Embedding {
                id: "1".to_string(),
                vector: vec![1.0, 2.0, 3.0],
                metadata: Some(metadata.clone())
            }],
        };
        db.collections.insert("test_collection".to_string(), collection);

        let new_embeddings = vec![
            Embedding {
                id: "1".to_string(), // Duplicate ID
                vector: vec![4.0, 5.0, 6.0],
                metadata: Some(metadata.clone())
            },
            Embedding {
                id: "2".to_string(),
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
        metadata.insert("page".to_string(), MetadataValue::Str("1".to_string()));
        metadata.insert("text".to_string(), MetadataValue::Str("This is a test metadata text".to_string()));
        let new_embeddings = vec![
            Embedding {
                id: "1".to_string(),
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

        let collection = Collection {
            dimension: 3,
            distance: Distance::Euclidean,
            embeddings: vec![
                Embedding { id: "1".to_string(), vector: vec![1.0, 1.0, 1.0], metadata: None },
                Embedding { id: "2".to_string(), vector: vec![2.0, 2.0, 2.0], metadata: None },
                Embedding { id: "3".to_string(), vector: vec![3.0, 3.0, 3.0], metadata: None },
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
        // Prepare a collection with embeddings for testing
        let collection = Collection {
            dimension: 3,
            distance: Distance::Euclidean,
            embeddings: vec![
                Embedding { id: "1".to_string(), vector: vec![1.0, 1.0, 1.0], metadata: None },
                Embedding { id: "2".to_string(), vector: vec![2.0, 2.0, 2.0], metadata: None },
                Embedding { id: "3".to_string(), vector: vec![3.0, 3.0, 3.0], metadata: None },
            ],
        };

        // Define a query vector
        let query = vec![0.0, 0.0, 0.0];

        // Define the expected similarity results
        let expected_results = vec![
            SimilarityResult { score: 0.0, embedding: Embedding { id: "2".to_string(), vector: vec![2.0, 2.0, 2.0], metadata: None } },
            SimilarityResult { score: 0.0, embedding: Embedding { id: "3".to_string(), vector: vec![3.0, 3.0, 3.0], metadata: None } },
            SimilarityResult { score: 0.0, embedding: Embedding { id: "1".to_string(), vector: vec![1.0, 1.0, 1.0], metadata: None } },
        ];

        // Call the get_similarity method
        let results = collection.get_similarity(&query, 3);

        // Assert that the results are as expected
        assert_eq!(results, expected_results);
    }

}