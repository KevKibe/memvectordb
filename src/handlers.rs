use warp::{Rejection, Reply, http::StatusCode, reply::json, reply::with_status, reply::WithStatus, reply::Json};
use crate::{
    model::{CacheDB, CreateCollectionStruct, InsertEmbeddingStruct, CollectionHandlerStruct, BatchInsertEmbeddingsStruct, GetSimilarityStruct},
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
                status: format!("Collection created: {:?}", collection_name),
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

pub async fn batch_insert_embeddings_handler(
    body: BatchInsertEmbeddingsStruct,
    db: Arc<Mutex<CacheDB>>,
) -> Result<impl Reply, Rejection> {
    let mut db_lock = db.lock().map_err(|_| warp::reject::reject())?;
    
    let result = db_lock.update_collection(&body.collection_name, body.embeddings);
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

pub async fn get_embeddings_handler(
    body: CollectionHandlerStruct,
    db: Arc<Mutex<CacheDB>>, 
) -> Result<impl Reply, Rejection> {
    let db_lock = db.lock().map_err(|_| warp::reject::reject())?;

    let embeddings = db_lock.get_embeddings(&body.collection_name);

    match embeddings {
        Some(embeddings) => {
            Ok(with_status(json(&embeddings), StatusCode::OK))
        }
        None => {
            let error_message = format!("Collection '{}' not found", body.collection_name);
            Ok(with_status(json(&error_message), StatusCode::NOT_FOUND))
        }
    }
}






#[cfg(test)]
mod tests {
    use super::*;
    use warp::http::StatusCode;
    use warp::Buf;
    use serde_json::{Value, json};
    use crate::model::{Distance, Embedding, SimilarityResult, CacheDB};
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_health_checker_handler() {
        let reply = health_checker_handler().await.unwrap();
        let response = reply.into_response();

        assert_eq!(response.status(), StatusCode::OK);

        let body = warp::hyper::body::aggregate(response.into_body()).await.unwrap();
        let body_value: Value = serde_json::from_reader(body.reader()).unwrap();
        let expected_body = json!({
            "status": "success",
            "message": "Health Check Sucessful!🚀"
        });
        assert_eq!(body_value, expected_body);
    }


    #[tokio::test]
    async fn test_create_collection_handler_success() {
        // Mocked request body
        let request_body = CreateCollectionStruct {
            collection_name: "test_collection".to_string(),
            dimension: 100,
            distance: Distance::Euclidean,
        };
    
        let db = Arc::new(Mutex::new(CacheDB::new()));
        let reply = create_collection_handler(request_body.clone(), db.clone()).await.unwrap();
        let response = reply.into_response();
    
        assert_eq!(response.status(), StatusCode::OK);
    
        let body = warp::hyper::body::aggregate(response.into_body()).await.unwrap();
        let body_value: Value = serde_json::from_reader(body.reader()).unwrap();
        let expected_response = json!({
            "result": "success",
            "status": "Collection created: \"test_collection\"",
        });
        assert_eq!(body_value, expected_response);
    
        // Verify that the collection was actually created in the database
        let db_lock = db.lock().unwrap();
        let collection = db_lock.get_collection(&request_body.collection_name).unwrap();
        assert_eq!(collection.dimension, request_body.dimension);
        assert_eq!(collection.distance, request_body.distance);
    }


    #[tokio::test]
    async fn test_insert_embeddings_handler_success() {

        let db = Arc::new(Mutex::new(CacheDB::new()));
        let request_body = CreateCollectionStruct {
            collection_name: "test_collection".to_string(),
            dimension: 3,
            distance: Distance::Euclidean,
        };
        let reply = create_collection_handler(request_body.clone(), db.clone()).await.unwrap();
        let response = reply.into_response();
    
        assert_eq!(response.status(), StatusCode::OK);
        let mut id = HashMap::new();
        id.insert("unique_id".to_string(), "1".to_string());
        let request_body = InsertEmbeddingStruct {
            collection_name: "test_collection".to_string(),
            embedding: Embedding { id: id, vector: vec![1.0, 1.0, 1.0], metadata: None },
        };
        let reply = insert_embeddings_handler(request_body.clone(), db.clone()).await.unwrap();
        let response = reply.into_response();
    
        assert_eq!(response.status(), StatusCode::OK);
    
        let body = warp::hyper::body::aggregate(response.into_body()).await.unwrap();
        let body_value: String = serde_json::from_reader(body.reader()).unwrap();
        let expected_response = format!("Embedding inserted into collection: {}", request_body.collection_name);
        assert_eq!(body_value, expected_response);
        
        let body = CollectionHandlerStruct {
            collection_name: "test_collection".to_string(),
        };
        let reply = get_embeddings_handler(body.clone(), db.clone()).await.unwrap();
        let response = reply.into_response();
        let mut body = warp::hyper::body::aggregate(response.into_body()).await.unwrap();
        let body_bytes = body.copy_to_bytes(body.remaining());
        let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();
        let embeddings: Vec<Embedding> = serde_json::from_str(&body_str).unwrap();
    
        assert_eq!(embeddings.len(), 1);
        assert_eq!(embeddings[0], request_body.embedding);


    }


    #[tokio::test]
    async fn test_get_collection_handler_success() {
        let db = Arc::new(Mutex::new(CacheDB::new()));
        let collection_name = "test_collection".to_string();

        let request_body = CreateCollectionStruct {
            collection_name: collection_name.clone(),
            dimension: 3,
            distance: Distance::Euclidean,
        };
        let _ = create_collection_handler(request_body.clone(), db.clone()).await.unwrap();

        let request_body = CollectionHandlerStruct {
            collection_name: collection_name.clone(),
        };
        let reply = get_collection_handler(request_body, db.clone()).await.unwrap();
        let response = reply.into_response();

        assert_eq!(response.status(), StatusCode::OK);
    }


    #[tokio::test]
    async fn test_get_collection_handler_not_found() {
        let db = Arc::new(Mutex::new(CacheDB::new()));
        let collection_name = "non_existent_collection".to_string();
        let request_body = CollectionHandlerStruct {
            collection_name: collection_name.clone(),
        };
        let reply = get_collection_handler(request_body, db.clone()).await.unwrap();
        let response = reply.into_response();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }


    #[tokio::test]
    async fn test_delete_collection_handler_success() {
        let db = Arc::new(Mutex::new(CacheDB::new()));
        let collection_name = "test_collection".to_string();
        let request_body = CreateCollectionStruct {
            collection_name: collection_name.clone(),
            dimension: 3,
            distance: Distance::Euclidean,
        };
        let _ = create_collection_handler(request_body.clone(), db.clone()).await.unwrap();

        let request_body = CollectionHandlerStruct {
            collection_name: collection_name.clone(),
        };
        let reply = delete_collection_handler(request_body, db.clone()).await.unwrap();
        let response = reply.into_response();

        assert_eq!(response.status(), StatusCode::OK);
    }


    #[tokio::test]
    async fn test_delete_collection_handler_not_found() {
        let db = Arc::new(Mutex::new(CacheDB::new()));
        let collection_name = "non_existent_collection".to_string();

        // Test delete_collection_handler
        let request_body = CollectionHandlerStruct {
            collection_name: collection_name.clone(),
        };
        let reply = delete_collection_handler(request_body, db.clone()).await.unwrap();
        let response = reply.into_response();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_update_collection_handler_success() {
        let db = Arc::new(Mutex::new(CacheDB::new()));
        let collection_name = "test_collection".to_string();

        // Insert a collection into the database
        let request_body = CreateCollectionStruct {
            collection_name: collection_name.clone(),
            dimension: 3,
            distance: Distance::Euclidean,
        };
        let _ = create_collection_handler(request_body.clone(), db.clone()).await.unwrap();

        let mut id_1 = HashMap::new();
        id_1.insert("unique_id".to_string(), "1".to_string());

        let mut id_2 = HashMap::new();
        id_2.insert("unique_id".to_string(), "2".to_string());

        // Update the collection
        let embeddings = vec![
            Embedding { id: id_1, vector: vec![2.0, 2.0, 2.0], metadata: None },
            Embedding { id: id_2, vector: vec![3.0, 3.0, 3.0], metadata: None },
        ];
        let request_body = BatchInsertEmbeddingsStruct {
            collection_name: collection_name.clone(),
            embeddings: embeddings.clone(),
        };
        let reply = batch_insert_embeddings_handler(request_body.clone(), db.clone()).await.unwrap();
        let response = reply.into_response();

        assert_eq!(response.status(), StatusCode::OK);

        let body = CollectionHandlerStruct {
            collection_name: "test_collection".to_string(),
        };
        let reply = get_embeddings_handler(body.clone(), db.clone()).await.unwrap();
        let response = reply.into_response();
        let mut body = warp::hyper::body::aggregate(response.into_body()).await.unwrap();
        let body_bytes = body.copy_to_bytes(body.remaining());
        let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();
        let _embeddings: Vec<Embedding> = serde_json::from_str(&body_str).unwrap();
    
        assert_eq!(_embeddings.len(), 2);
        assert_eq!(_embeddings[0], embeddings[0]);
    }

    #[tokio::test]
    async fn test_update_collection_handler_not_found() {
        let db = Arc::new(Mutex::new(CacheDB::new()));
        let collection_name = "non_existent_collection".to_string();

        let mut id_1 = HashMap::new();
        id_1.insert("unique_id".to_string(), "1".to_string());

        let mut id_2 = HashMap::new();
        id_2.insert("unique_id".to_string(), "2".to_string());

        // Try to update a non-existent collection
        let embeddings = vec![
            Embedding { id: id_1, vector: vec![2.0, 2.0, 2.0], metadata: None },
            Embedding { id: id_2, vector: vec![3.0, 3.0, 3.0], metadata: None },
        ];
        let request_body = BatchInsertEmbeddingsStruct {
            collection_name: collection_name.clone(),
            embeddings: embeddings.clone(),
        };
        let reply = batch_insert_embeddings_handler(request_body, db.clone()).await.unwrap();
        let response = reply.into_response();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }


    #[tokio::test]
    async fn test_get_similarity_handler_success() {
        let db = Arc::new(Mutex::new(CacheDB::new()));
        let collection_name = "test_collection".to_string();

        // Insert a collection into the database
        let request_body = CreateCollectionStruct {
            collection_name: collection_name.clone(),
            dimension: 3,
            distance: Distance::Euclidean,
        };
        let _ = create_collection_handler(request_body.clone(), db.clone()).await.unwrap();

        let mut metadata = HashMap::new();
        metadata.insert("page".to_string(), "1".to_string());
        metadata.insert("text".to_string(), "This is a test metadata text".to_string());

        let mut id = HashMap::new();
        id.insert("unique_id".to_string(), "0".to_string());

        // Insert an embedding into the collection
        let embedding = Embedding { id: id, vector: vec![1.0, 1.0, 1.0], metadata: Some(metadata.clone())};
        
        let insert_request_body = InsertEmbeddingStruct {
            collection_name: collection_name.clone(),
            embedding: embedding.clone(),
        };
        let _ = insert_embeddings_handler(insert_request_body.clone(), db.clone()).await.unwrap();

        // Test get_similarity_handler
        let request_body = GetSimilarityStruct {
            collection_name: collection_name.clone(),
            query_vector: vec![1.0, 1.0, 1.0],
            k: 1,
        };
        let reply = get_similarity_handler(request_body, db.clone()).await.unwrap();
        let response = reply.into_response();
        let mut body = warp::hyper::body::aggregate(response.into_body()).await.unwrap();
        let body_bytes = body.copy_to_bytes(body.remaining());
        let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();
        let similarity_results: Vec<SimilarityResult> = serde_json::from_str(&body_str).unwrap();

        assert_eq!(similarity_results.len(), 1);
        assert_eq!(similarity_results[0].score, 0.0);
        assert_eq!(similarity_results[0].embedding, embedding);
        assert_eq!(similarity_results[0].embedding.metadata, Some(metadata.clone()));
    }

    #[tokio::test]
    async fn test_get_similarity_handler_not_found() {
        let db = Arc::new(Mutex::new(CacheDB::new()));
        let collection_name = "non_existent_collection".to_string();

        // Test get_similarity_handler
        let request_body = GetSimilarityStruct {
            collection_name: collection_name.clone(),
            query_vector: vec![1.0, 1.0, 1.0],
            k: 1,
        };
        let reply = get_similarity_handler(request_body, db.clone()).await.unwrap();
        let response = reply.into_response();
        let body = warp::hyper::body::aggregate(response.into_body()).await.unwrap();
        let body_value: Value = serde_json::from_reader(body.reader()).unwrap();

        assert_eq!(body_value, "Collection not found");
    }

    #[tokio::test]
    async fn test_get_embeddings_handler_success() {
        let db = Arc::new(Mutex::new(CacheDB::new()));
        let collection_name = "test_collection".to_string();

        let request_body = CreateCollectionStruct {
            collection_name: collection_name.clone(),
            dimension: 3,
            distance: Distance::Euclidean,
        };
        let _ = create_collection_handler(request_body.clone(), db.clone()).await.unwrap();

        let request_body = CollectionHandlerStruct {
            collection_name: collection_name.clone(),
        };
        let reply = get_embeddings_handler(request_body, db.clone()).await.unwrap();
        let response = reply.into_response();

        assert_eq!(response.status(), StatusCode::OK);
    }


    #[tokio::test]
    async fn test_get_embeddings_handler_not_found() {
        let db = Arc::new(Mutex::new(CacheDB::new()));
        let collection_name = "non_existent_collection".to_string();
        let request_body = CollectionHandlerStruct {
            collection_name: collection_name.clone(),
        };
        let reply = get_embeddings_handler(request_body, db.clone()).await.unwrap();
        let response = reply.into_response();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

}



