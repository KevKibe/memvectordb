use std::fs::File;
use std::io::{BufReader, BufRead};
use regex::Regex;
use std::error::Error;
use crate::model::{CacheDB, Distance, Embedding};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

pub fn replay_logs() -> Result<(), String> {
    let db = Arc::new(Mutex::new(CacheDB::new()));
    let file = File::open("output.log").map_err(|e| e.to_string())?;
    let reader = BufReader::new(file);


    let mut log_content = String::new();
    for line in reader.lines() {
        let line = line.map_err(|e| e.to_string())?;
        log_content.push_str(&line);
    }

    let log_entries = split_by_date(&log_content);

    for cleaned_entry in log_entries {
        if cleaned_entry.contains("Created new collection") {
            parse_and_create_collection(&cleaned_entry, db.clone());
        }
        else if cleaned_entry.contains("successfully inserted into collection") {
            // println!("{}", cleaned_entry);
            // parse_and_insert_embeddings(&cleaned_entry, db.clone());
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
    print!("{}", log_line);
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


// pub fn parse_and_insert_embeddings(log_line: &str, db: Arc<Mutex<CacheDB>>) -> Result<(), Box<dyn Error>> {
//     let re = Regex::new(
//         r#"Embedding: 'Embedding \{ id: \{"unique_id": "(\d+)"\}, vector: \[([0-9.,\s]+)\], metadata: Some\{([^}]*)\}\}', successfully inserted into collection '([^']*)'"#,
//     )?;
//     println!("{}", re);

//     if let Some(caps) = re.captures(log_line) {
//         let collection_name = caps.get(4).map_or("", |m| m.as_str()).to_string();
//         println!("Collection Name: {}", collection_name);
    
//         let vector: Vec<f32> = caps.get(2)
//             .map_or("", |m| m.as_str())
//             .split(',')
//             .filter_map(|s| s.trim().parse().ok())
//             .collect();
    
//         let metadata = caps.get(3).map_or(None, |m| {
//             let metadata_str = m.as_str();
//             if metadata_str.is_empty() {
//                 None
//             } else {
//                 Some(metadata_str
//                     .split(',')
//                     .map(|entry| {
//                         let mut kv = entry.trim().split(':');
//                         let key = kv.next().unwrap_or("").trim().to_string();
//                         let value = kv.next().unwrap_or("").trim().to_string();
//                         (key, value)
//                     })
//                     .collect())
//             }
//         });
    
//         let embedding = Embedding {
//             id: HashMap::new(),
//             vector,
//             metadata,
//         };

//         println!("{:?}", embedding);
//         // let mut db = db.lock().map_err(|e| format!("Failed to lock the database: {}", e))?;
//         // db.insert_into_collection(&collection_name, embedding)?;
//     } else {
//         eprintln!("Log line format is incorrect: {}", log_line);
//     }

//     Ok(())
// }
