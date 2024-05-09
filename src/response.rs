use serde::Serialize;

#[derive(Serialize)]
pub struct CreateCollectionResponse {
    pub result: String,
    pub status: String,
}

#[derive(Serialize)]
pub struct GenericResponse {
    pub status: String,
    pub message: String,
}
