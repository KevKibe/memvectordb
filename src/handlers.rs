use warp::{Rejection, Reply, http::StatusCode, reply::json, reply::with_status, reply::WithStatus, reply::Json};
use crate::{
    model::{CacheDB, CreateCollectionStruct, InsertEmbeddingStruct, CollectionHandlerStruct, UpdateCollectionStruct, GetSimilarityStruct},
    response::{CreateCollectionResponse, GenericResponse},
    WebResult
};
use std::sync::{Arc, Mutex};


pub async fn health_checker_handler() -> WebResult<impl Reply> {
    const MESSAGE: &str = "Health Check Sucessful!🚀";

    let response_json = &GenericResponse {
        status: "success".to_string(),
        message: MESSAGE.to_string(),
    };
    Ok(json(response_json))
}

pub async fn create_collection_handler(
    body: CreateCollectionStruct,
    db: Arc<Mutex<CacheDB>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let collection_name = body.collection_name;
    let dimension = body.dimension;
    let distance = body.distance;
    let mut db_lock = db.lock().map_err(|_| warp::reject::reject())?;
    match db_lock.create_collection(collection_name.clone(), dimension, distance) {
        Ok(collection) => {
            println!("Successfully created collection: {:?}", collection);
            Ok(json(&CreateCollectionResponse {
                result: "success".to_string(),
                status: "Collection created".to_string(),
            }))
        }
        Err(err) => {
            println!("Failed to create collection: {:?}", err);
            Ok(json(&CreateCollectionResponse {
                result: "failure".to_string(),
                status: format!("Error: {:?}", err),
            }))
        }
    }
}

pub async fn insert_embeddings_handler(
    body: InsertEmbeddingStruct,
    db: Arc<Mutex<CacheDB>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut db_lock = db.lock().map_err(|_| warp::reject::reject())?;

    let result = db_lock.insert_into_collection(&body.collection_name, body.embedding);

    match result {
        Ok(_) => {
            println!("Successfully inserted embedding into collection: {}", &body.collection_name);
            Ok(warp::reply::json(&format!("Embedding inserted into collection: {}", &body.collection_name)))
        }
        Err(err) => {
            eprintln!("Failed to insert embedding into collection: {}. Error: {:?}", &body.collection_name, err);
            Ok(warp::reply::json(&format!("Failed to insert embedding into collection: {}. Error: {:?}", &body.collection_name, err)))
        }
    }
}


pub async fn get_collection_handler(
    body: CollectionHandlerStruct,
    db: Arc<Mutex<CacheDB>>,
) -> Result<WithStatus<Json>, Rejection> {
    let db_lock = db.lock().map_err(|_| warp::reject::reject())?;
    
    let collection = db_lock.get_collection(&body.collection_name);

    match collection {
        Some(collection) => {
            Ok(with_status(json(&collection), StatusCode::OK))
        }
        None => {
            let error_message = format!("Collection '{}' not found", body.collection_name);
            Ok(with_status(json(&error_message), StatusCode::NOT_FOUND))
        }
    }
}

pub async fn delete_collection_handler(
    body: CollectionHandlerStruct,
    db: Arc<Mutex<CacheDB>>,
) -> Result<impl Reply, Rejection> {
    let mut db_lock = db.lock().map_err(|_| warp::reject::reject())?;
    
    let result = db_lock.delete_collection(&body.collection_name);

    match result {
        Ok(_) => {
            let success_message = format!("Collection '{}' deleted successfully", body.collection_name);
            Ok(with_status(json(&success_message), StatusCode::OK))
        }
        Err(err) => {
            let error_message = format!("Failed to delete collection '{}': {:?}", body.collection_name, err);
            Ok(with_status(json(&error_message), StatusCode::NOT_FOUND))
        }
    }
}

pub async fn update_collection_handler(
    body: UpdateCollectionStruct,
    db: Arc<Mutex<CacheDB>>,
) -> Result<impl Reply, Rejection> {
    let mut db_lock = db.lock().map_err(|_| warp::reject::reject())?;
    
    let result = db_lock.update_collection(&body.collection_name, body.new_embeddings);
    match result {
        Ok(_) => {
            let success_message = format!("Collection '{}' updated successfully", body.collection_name);
            Ok(with_status(json(&success_message), StatusCode::OK))
        }
        Err(err) => {
            let error_message = format!("Failed to update collection '{}': {:?}", body.collection_name, err);
            Ok(with_status(json(&error_message), StatusCode::NOT_FOUND))
        }
    }
}



pub async fn get_similarity_handler(
    body: GetSimilarityStruct,
    db: Arc<Mutex<CacheDB>>,
) -> Result<impl Reply, Rejection> {
    let db_lock = db.lock().map_err(|_| warp::reject::reject())?;

    if let Some(collection) = db_lock.get_collection(&body.collection_name) {
        let similarity_results = collection.get_similarity(&body.query_vector, body.k);
        return Ok(json(&similarity_results));
    }

    Ok(json(&"Collection not found"))
}
