use axum::{body::Body, response::Response};
use grammers_client::{Client, grammers_tl_types as tl};

use crate::{proto::FileRef, shared::CHUNK_SIZE};

use super::refresh_file_reference;

pub async fn get_chunk(client: &Client, file_ref: String, offset: usize) -> Response {
    let file_ref = FileRef::from_ref_string(file_ref);
    let file_ref = match file_ref {
        Some(v) => v,
        None => {
            return Response::builder().status(404).body(Body::from("chunk not found")).unwrap();
        }
    };
    let file_ref = match file_ref.v1 {
        Some(v) => v,
        None => {
            return Response::builder().status(404).body(Body::from("chunk not found")).unwrap();
        }
    };

    if (offset % CHUNK_SIZE) > 0 {
        return Response::builder().status(400).body(Body::from("offset should be divisible by 524288")).unwrap();
    }

    let req = tl::functions::upload::GetFile {
        cdn_supported: false,
        limit: CHUNK_SIZE as i32,
        location: tl::enums::InputFileLocation::InputDocumentFileLocation(tl::types::InputDocumentFileLocation {
            id: file_ref.document_id, access_hash: file_ref.access_hash, file_reference: file_ref.file_reference.clone(), thumb_size: "".to_string()
        }),
        precise: false,
        offset: offset as _,
    };

    let res = client.invoke(&req).await;

    let res = match res {
        Ok(v) => v,
        Err(e) => {
            match &e {
                grammers_client::InvocationError::Rpc(e) => {
                    if e.name == "FILE_REFERENCE_EXPIRED" {
                        match refresh_file_reference(client, &file_ref).await {
                            None => {
                                return Response::builder().status(404).body(Body::from("chunk not found")).unwrap();
                            }
                            Some(v) => {
                                return Response::builder()
                                    .status(409)
                                    .header("X-New-Ref", v)
                                    .body(Body::empty())
                                .unwrap();
                            }
                        }
                    }
                },
                _ => {},
            };
            println!("failed to get file {:?}", e);
            return Response::builder().status(500).body(Body::from("failed to fetch from upstream")).unwrap();
        }
    };

    let res = match res {
        tl::enums::upload::File::File(file) => file,
        tl::enums::upload::File::CdnRedirect(file_cdn_redirect) => {
            println!("TODO: redirected to cdn {:?}", file_cdn_redirect);
            return Response::builder().status(500).body(Body::from("failed to fetch from upstream")).unwrap();
        },
    };

    Response::builder()
        .status(200)
        .header("Content-Type", "application/octet-stream")
        .body(Body::from(res.bytes))
    .unwrap()
}