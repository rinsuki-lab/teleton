use axum::{body::Body, response::Response};
use rand::{rngs::StdRng, RngCore, SeedableRng};

use crate::{proto::{UploadToken, UploadTokenV1}, shared::CHUNK_SIZE};

#[derive(serde::Deserialize)]
pub struct StartUploadQueryParams {
    file_size: u64,
}

#[derive(serde::Serialize)]
pub struct StartUploadResponse {
    token: String,
    chunk_size: usize,
}

pub async fn start_upload(query: StartUploadQueryParams) -> Response {
    let file_id = StdRng::from_entropy().next_u64();

    let token = UploadTokenV1 {
        file_id: (file_id as i64).abs(),
        file_size: query.file_size as i64,
    };

    let body = StartUploadResponse {
        token: UploadToken { v1: Some(token) }.to_api_string(),
        chunk_size: CHUNK_SIZE,
    };

    let body = serde_json::to_vec(&body).unwrap();

    Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .body(Body::from(body))
    .unwrap()
}