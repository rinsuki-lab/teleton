use axum::{body::Body, response::Response};
use grammers_client::{Client, grammers_tl_types as tl};

use crate::{proto::UploadToken, shared::CHUNK_SIZE};

#[derive(serde::Deserialize)]
pub struct UploadChunkQueryParams {
    token: String,
    offset: u64,
}

pub async fn upload_chunk(client: &Client, query: UploadChunkQueryParams, body: Vec<u8>) -> Response {
    let token = UploadToken::from_api_string(query.token);

    let token = match token {
        None => {
            return Response::builder().status(400).body(Body::from("invalid token")).unwrap();
        }
        Some(t) => t,
    };

    let token = match token.v1 {
        None => {
            return Response::builder().status(400).body(Body::from("invalid token")).unwrap();
        }
        Some(t) => t,
    };

    if query.offset % (CHUNK_SIZE as u64) > 0 {
        return Response::builder().status(400).body(Body::from(format!("offset should be divided by {}", CHUNK_SIZE))).unwrap();
    }

    let current_part = (query.offset / (CHUNK_SIZE as u64)) as i32;
    println!("{}, {}", current_part, query.offset);

    let res = if token.should_use_big_upload() {
        // big
        let req = tl::functions::upload::SaveBigFilePart {
            bytes: body,
            file_id: token.file_id,
            file_part: current_part,
            file_total_parts: token.total_parts(),
        };
        client.invoke(&req).await
    } else {
        // small
        let req = tl::functions::upload::SaveFilePart {
            bytes: body,
            file_id: token.file_id,
            file_part: current_part,
        };
        client.invoke(&req).await
    };

    match res {
        Ok(v) => {
            println!("{}", v);
        },
        Err(e) => {
            println!("failed to call upstream api {:?}", e);
            return Response::builder().status(400).body(Body::from("failed to call upstream api")).unwrap();
        }
    }

    Response::builder().status(204).body(Body::empty()).unwrap()
}