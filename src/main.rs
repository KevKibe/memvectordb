mod db;
mod similarity;
mod handlers;
mod model;
mod response;

use handlers::{
    health_checker_handler, 
    create_collection_handler, 
    insert_embeddings_handler, 
    get_collection_handler, 
    delete_collection_handler, 
    update_collection_handler, 
    get_similarity_handler
};
use warp::{Filter,Rejection};
use crate::model::{
    CacheDB, 
    CreateCollectionStruct, 
    InsertEmbeddingStruct, 
    CollectionHandlerStruct, 
    UpdateCollectionStruct, 
    GetSimilarityStruct
};
use std::sync::{Arc, Mutex};

type WebResult<T> = std::result::Result<T, Rejection>;


#[tokio::main]
async fn main() {
    // Create a shared CacheDB instance wrapped in Mutex and Arc
    let db = Arc::new(Mutex::new(CacheDB::new()));

    let health_checker_route = warp::path!("healthchecker")
        .and(warp::get())
        .and_then(health_checker_handler);

    // Define the filter to inject the shared CacheDB instance into request handlers
    let with_db = warp::any().map(move || db.clone());

    let create_collection_route = warp::path!("create_collection")
        .and(warp::post())
        .and(warp::body::json::<CreateCollectionStruct>())
        .and(with_db.clone())  
        .and_then(create_collection_handler);

    let insert_embeddings_route = warp::path!("insert_embeddings")
        .and(warp::post())
        .and(warp::body::json::<InsertEmbeddingStruct>())
        .and(with_db.clone())  
        .and_then(insert_embeddings_handler);

    let get_collection_route = warp::path!("get_collection")
        .and(warp::post())
        .and(warp::body::json::<CollectionHandlerStruct>())
        .and(with_db.clone())  
        .and_then(get_collection_handler);

    let delete_collection_route = warp::path!("delete_collection")
        .and(warp::post())
        .and(warp::body::json::<CollectionHandlerStruct>())
        .and(with_db.clone())
        .and_then(delete_collection_handler);

    let update_collection_route = warp::path!("update_collection")
        .and(warp::post())
        .and(warp::body::json::<UpdateCollectionStruct>())
        .and(with_db.clone())
        .and_then(update_collection_handler);

    let get_similarity_route = warp::path!("get_similarity")
        .and(warp::post())
        .and(warp::body::json::<GetSimilarityStruct>())
        .and(with_db.clone())
        .and_then(get_similarity_handler);
    // Combine the routes
    let routes = health_checker_route
        .or(create_collection_route)
        .or(insert_embeddings_route)
        .or(get_collection_route)
        .or(delete_collection_route)
        .or(update_collection_route)
        .or(get_similarity_route);

    // Start the server
    println!("🚀 Server started successfully");
    warp::serve(routes)
        .run(([0, 0, 0, 0], 8000))
        .await;
}

