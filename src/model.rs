use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use schemars::JsonSchema;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CacheDB {
	pub collections: HashMap<String, Collection>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, JsonSchema, PartialEq)]
pub struct SimilarityResult {
	pub score: f32,
	pub embedding: Embedding,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, JsonSchema, PartialEq)]
pub struct Collection {
	pub dimension: usize,
	pub distance: Distance,
	#[serde(default)]
	pub embeddings: Vec<Embedding>,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, JsonSchema, PartialEq)]
pub struct Embedding {
	pub id: String,
	pub vector: Vec<f32>,
	pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum Distance {
	#[serde(rename = "euclidean")]
	Euclidean,
	#[serde(rename = "cosine")]
	Cosine,
	#[serde(rename = "dot")]
	DotProduct,
}



#[derive(Debug, thiserror::Error, PartialEq)]
pub enum Error {
	#[error("Collection already exists")]
	UniqueViolation,

	#[error("Collection doesn't exist")]
	NotFound,

	#[error("The dimension of the vector doesn't match the dimension of the collection")]
	DimensionMismatch,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, JsonSchema)]
pub struct CreateCollectionStruct{
    pub collection_name: String,
    pub dimension: usize,
    pub distance: Distance,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, JsonSchema)]

pub struct InsertEmbeddingStruct{
	pub collection_name: String,
	pub embedding: Embedding,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CollectionHandlerStruct{
	pub collection_name: String,
}


#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, JsonSchema)]

pub struct UpdateCollectionStruct{
	pub collection_name: String,
	pub new_embeddings: Vec<Embedding>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, JsonSchema)]
pub struct GetSimilarityStruct{
	pub collection_name: String,
	pub query_vector: Vec<f32>,
	pub k: usize
}