use rayon::prelude::*;
use std::{
    collections::{BinaryHeap, HashMap},
};
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
}
