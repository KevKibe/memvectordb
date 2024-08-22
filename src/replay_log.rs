use std::fs::File;
use std::io::{BufReader, BufRead};
use regex::Regex;
use std::error::Error;
use crate::model::{CacheDB, Distance, Embedding};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

pub fn restore_db_from_logs(db: Arc<Mutex<CacheDB>>) -> Result<(), String> {
    // let db = Arc::new(Mutex::new(CacheDB::new()));
    let file = File::open("output.log").map_err(|e| e.to_string())?;
    let reader = BufReader::new(file);


    let mut log_content = String::new();
    for line in reader.lines() {
        let line = line.map_err(|e| e.to_string())?;
        log_content.push_str(&line);
    }

    let log_entries = split_by_date(&log_content);

    for entry in log_entries {
        if entry.contains("Created new collection") {
            let _restored_db = parse_and_create_collection(&entry, db.clone());
        }
        else if entry.contains("successfully inserted into collection") {
            let _restored_db = parse_and_insert_embeddings(&entry, db.clone());
        }
        else if entry.contains("successfully updated to collection") {
            let _restored_db = parse_and_update_collection(&entry, db.clone());
        }
        else if entry.contains("Deleted collection") {
            let _restored_db = parse_and_delete_collection(&entry, db.clone());
        }
    }
    Ok(())
}

fn split_by_date(log: &str) -> Vec<String> {
    let re = Regex::new(r"\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}").unwrap();
    let mut entries: Vec<String> = Vec::new();
    let mut start = 0;
    for mat in re.find_iter(log) {
        let end = mat.start();
        if start != end {
            entries.push(log[start..end].trim().to_string());
        }

        start = end;
    }
    if start < log.len() {
        entries.push(log[start..].trim().to_string());
    }

    entries
}

pub fn parse_and_create_collection(log_line :&str, db: Arc<Mutex<CacheDB>>) -> Result<(), Box<dyn Error>> {
    let re = Regex::new(
        r"Created new collection with name: '([^']+)', dimension: '(\d+)', distance: '([^']+)'",
    )?;

    if let Some(caps) = re.captures(log_line) {
        let collection_name = caps.get(1).unwrap().as_str().to_string();
        let collection_dimension: usize = caps.get(2).unwrap().as_str().parse()?;
        let collection_distance = caps.get(3).unwrap().as_str();

        let distance = match collection_distance {
            "DotProduct" => Distance::DotProduct,
            "Cosine" => Distance::Cosine,
            "Euclidean" => Distance::Euclidean,
            _ => return Err("Unknown distance type".into()),
        };

        let mut db = db.lock().unwrap();
        db.create_collection(collection_name, collection_dimension, distance)?;
    }
    else {
        eprintln!("Log line format is incorrect: {}", log_line);
    }
    
    Ok(())
}


pub fn parse_and_insert_embeddings(log_line: &str, db: Arc<Mutex<CacheDB>>) -> Result<(), Box<dyn Error>> {
    let re = Regex::new(
        r#"Embedding: 'Embedding \{ id: \{"unique_id": "(\d+)"\}, vector: \[([0-9.,\s]+)\], metadata: Some\(\{(.*?)\}\) \}', successfully inserted into collection '([^']*)'"#
    )?;

    if let Some(caps) = re.captures(log_line) {
        let collection_name = caps.get(4).map_or("", |m| m.as_str()).to_string();
        
        let vector: Vec<f32> = caps.get(2)
            .map_or("", |m| m.as_str())
            .split(',')
            .filter_map(|s| s.trim().parse().ok())
            .collect();

        let metadata = caps.get(3).map(|m| {
            let metadata_str = m.as_str();
            metadata_str
                .split(',')
                .map(|entry| {
                    let mut kv = entry.splitn(2, ':');
                    let key = kv.next().unwrap_or("").trim().trim_matches('"').to_string();
                    let value = kv.next().unwrap_or("").trim().trim_matches('"').to_string();
                    (key, value)
                })
                .collect::<HashMap<String, String>>()  
        });

        let unique_id = caps.get(1).map_or("", |m| m.as_str()).to_string();
        let mut id = HashMap::new();
        id.insert("unique_id".to_string(), unique_id);

        let embedding = Embedding {
            id,
            vector,
            metadata,
        };

        let mut db = db.lock().map_err(|e| format!("Failed to lock the database: {}", e))?;
        db.insert_into_collection(&collection_name, embedding)?;
    } 
    else {
        eprintln!("Log line format is incorrect: {}", log_line);
    }

    Ok(())
}

pub fn parse_and_update_collection(log_line: &str, db: Arc<Mutex<CacheDB>>) -> Result<(), Box<dyn Error>> {
    let re = Regex::new(
        r#"Embedding: '\[(.*?)\]' successfully updated to collection '([^']*)'"#
    )?;    

    if let Some(caps) = re.captures(log_line) {
        let embeddings_str = caps.get(1).map_or("", |m| m.as_str());
        let collection_name = caps.get(2).map_or("", |m| m.as_str()).to_string();

        // Regex to capture individual embeddings within the list
        let embedding_re = Regex::new(
            r#"Embedding \{ id: \{"unique_id": "(\d+)"\}, vector: \[([0-9.,\s]+)\], metadata: Some\(\{(.*?)\}\) \}"#
        )?;

        let mut new_embeddings = Vec::new();

        // Iterate over each match for individual embeddings
        for cap in embedding_re.captures_iter(embeddings_str) {
            let unique_id = cap.get(1).map_or("", |m| m.as_str()).to_string();
            let vector: Vec<f32> = cap.get(2)
                .map_or("", |m| m.as_str())
                .split(',')
                .filter_map(|s| s.trim().parse().ok())
                .collect();

            let metadata = cap.get(3).map(|m| {
                let metadata_str = m.as_str();
                metadata_str
                    .split(',')
                    .map(|entry| {
                        let mut kv = entry.splitn(2, ':');
                        let key = kv.next().unwrap_or("").trim().trim_matches('"').to_string();
                        let value = kv.next().unwrap_or("").trim().trim_matches('"').to_string();
                        (key, value)
                    })
                    .collect::<HashMap<String, String>>()
            });

            let mut id = HashMap::new();
            id.insert("unique_id".to_string(), unique_id);

            new_embeddings.push(Embedding {
                id,
                vector,
                metadata,
            });
        }

        let mut db = db.lock().map_err(|e| format!("Failed to lock the database: {}", e))?;
        db.update_collection(&collection_name, new_embeddings)?;
    } 
    else {
        eprintln!("Log line format is incorrect: {}", log_line);
    }

    Ok(())
}


pub fn parse_and_delete_collection(log_line: &str, db: Arc<Mutex<CacheDB>>) -> Result<(), Box<dyn Error>> {
    let re = Regex::new(r#"Deleted collection: '([^']*)'"#)?;

    if let Some(caps) = re.captures(log_line) {
        let collection_name = caps.get(1).map_or("", |m| m.as_str());

        let mut db = db.lock().map_err(|e| format!("Failed to lock the database: {}", e))?;
        db.delete_collection(&collection_name)?;

    } else {
        eprintln!("Log line format is incorrect: {}", log_line);
    }
    Ok(())
}