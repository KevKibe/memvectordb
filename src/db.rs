use axum::Extension;
use rayon::prelude::*;
use std::{
	collections::{BinaryHeap, HashMap},
	sync::Arc,
};
use tokio::sync::RwLock;
use crate::similarity::{get_cache_attr, get_distance_fn, normalize, ScoreIndex};
use crate::model::{CacheDB, SimilarityResult, Collection, Embedding, Distance, Error};


#[allow(clippy::module_name_repetitions)]
pub type DbExtension = Extension<Arc<RwLock<CacheDB>>>;

impl Collection {
	pub fn get_similarity(&self, query: &[f32], k: usize) -> Vec<SimilarityResult> {
		let memo_attr = get_cache_attr(self.distance, query);
		let distance_fn = get_distance_fn(self.distance);

		let scores = self
			.embeddings
			.par_iter()
			.enumerate()
			.map(|(index, embedding)| {
				let score = distance_fn(&embedding.vector, query, memo_attr);
				ScoreIndex { score, index }
			})
			.collect::<Vec<_>>();

		let mut heap = BinaryHeap::new();
		for score_index in scores {
			if heap.len() < k || score_index < *heap.peek().unwrap() {
				heap.push(score_index);

				if heap.len() > k {
					heap.pop();
				}
			}
		}

		heap.into_sorted_vec()
			.into_iter()
			.map(|ScoreIndex { score, index }| SimilarityResult {
				score,
				embedding: self.embeddings[index].clone(),
			})
			.collect()
	}
}


impl CacheDB {
	pub fn new() -> Self {
		Self {
			collections: HashMap::new(),
		}
	}

	pub fn extension(self) -> DbExtension {
		Extension(Arc::new(RwLock::new(self)))
	}

	pub fn create_collection(
		&mut self,
		name: String,
		dimension: usize,
		distance: Distance,
	) -> Result<Collection, Error> {
		if self.collections.contains_key(&name) {
			return Err(Error::UniqueViolation);
		}

		let collection = Collection {
			dimension,
			distance,
			embeddings: Vec::new(),
		};

		self.collections.insert(name, collection.clone());

		Ok(collection)
	}

	pub fn delete_collection(&mut self, name: &str) -> Result<(), Error> {
		if !self.collections.contains_key(name) {
			return Err(Error::NotFound);
		}

		self.collections.remove(name);

		Ok(())
	}

	pub fn insert_into_collection(
		&mut self,
		collection_name: &str,
		mut embedding: Embedding,
	) -> Result<(), Error> {
		let collection = self
			.collections
			.get_mut(collection_name)
			.ok_or(Error::NotFound)?;

		if collection.embeddings.iter().any(|e| e.id == embedding.id) {
			return Err(Error::UniqueViolation);
		}

		if embedding.vector.len() != collection.dimension {
			return Err(Error::DimensionMismatch);
		}

		// Normalize the vector if the distance metric is cosine, so we can use dot product later
		if collection.distance == Distance::Cosine {
			embedding.vector = normalize(&embedding.vector);
		}

		collection.embeddings.push(embedding);

		Ok(())
	}

    pub fn update_collection(
        &mut self,
        collection_name: &str,
        new_embeddings: Vec<Embedding>,
    ) -> Result<(), Error> {
        // Get a mutable reference to the specified collection.
        let collection = self
            .collections
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

            // Normalize the vector if the distance metric is cosine, so we can use dot product later.
            if collection.distance == Distance::Cosine {
                embedding.vector = normalize(&embedding.vector);
            }

            // Add the embedding to the collection.
            collection.embeddings.push(embedding);
        }

        Ok(())
    }

	pub fn get_collection(&self, name: &str) -> Option<&Collection> {
		self.collections.get(name)
	}
}

