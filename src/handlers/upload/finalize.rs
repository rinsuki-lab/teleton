use axum::{body::Body, response::Response};
use base64::Engine;
use grammers_client::{Client, grammers_tl_types as tl};
use prost::Message;

use crate::proto::{FileRef, FileRefV1, UploadToken};

#[derive(serde::Deserialize)]
pub struct UploadFinalizeQueryParams {
    token: String
}

#[derive(serde::Deserialize)]
pub struct UploadFinalizeBody {
    md5: String,
    name: String,
}

#[derive(serde::Serialize)]
pub struct UploadFinalizeResponse {
    r#ref: String,
}

pub async fn upload_finalize(query: UploadFinalizeQueryParams, body: UploadFinalizeBody, client: &Client) -> Response {
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

    let file: tl::enums::InputFile = match token.should_use_big_upload() {
        true => {
            tl::enums::InputFile::Big(tl::types::InputFileBig {
                id: token.file_id,
                parts: token.total_parts(),
                name: body.name,
            })
        },
        false => {
            tl::enums::InputFile::File(tl::types::InputFile {
                id: token.file_id,
                md5_checksum: body.md5,
                name: body.name,
                parts: token.total_parts(),
            })
        }
    };

    let req = tl::functions::messages::SendMedia {
        silent: true,
        background: false,
        clear_draft: false,
        noforwards: true,
        update_stickersets_order: false,
        invert_media: false,
        peer: tl::enums::InputPeer::PeerSelf,
        reply_to: None,
        media: tl::enums::InputMedia::UploadedDocument(tl::types::InputMediaUploadedDocument {
            nosound_video: false,
            force_file: true,
            spoiler: false,
            file,
            thumb: None,
            mime_type: "application/octet-stream".to_string(),
            attributes: vec![],
            stickers: None,
            ttl_seconds: None,
        }),
        message: "".to_string(),
        random_id: token.file_id,
        reply_markup: None,
        entities: None,
        schedule_date: None,
        send_as: None,
        quick_reply_shortcut: None,
        effect: None,
    };

    let res = client.invoke(&req).await;

    let res = match res {
        Ok(v) => v,
        Err(e) => {
            println!("failed to send message to upstream {:?}", e);
            return Response::builder().status(400).body(Body::from("failed to call upstream api")).unwrap();
        }
    };

    let res = match res {
        tl::enums::Updates::Updates(updates) => updates,
        _ => {
            println!("upstream returns unexpected updates {:?}", res);
            return Response::builder().status(500).body(Body::from("failed to call upstream api")).unwrap();
        }
    };

    let res = match res.updates.iter().find_map(|u| {
        match u {
            tl::enums::Update::NewMessage(m) => Some(m),
            _ => None,
        }
    }) {
        Some(v) => v,
        None => {
            println!("upstream doesn't return NewMessage in updates {:?}", res.updates);
            return Response::builder().status(500).body(Body::from("failed to call upstream api")).unwrap();
        }
    };

    let res = match &res.message {
        tl::enums::Message::Message(message) => message,
        _ => {
            println!("upstream doesn't return Message {:?}", res.message);
            return Response::builder().status(500).body(Body::from("failed to call upstream api")).unwrap();
        }
    };

    let doc = match &res.media {
        None => {
            println!("upstream doesn't contains media in message {:?}", res);
            return Response::builder().status(500).body(Body::from("failed to call upstream api")).unwrap();
        },
        Some(v) => v,
    };
    let doc = match doc {
        tl::enums::MessageMedia::Document(d) => d,
        _ => {
            println!("upstream media isn't document {:?}", doc);
            return Response::builder().status(500).body(Body::from("failed to call upstream api")).unwrap();
        }
    };
    let doc = match &doc.document {
        Some(v) => v,
        None => {
            println!("upstream document isn't available {:?}", doc);
            return Response::builder().status(500).body(Body::from("failed to call upstream api")).unwrap();
        },
    };
    let doc = match doc {
        tl::enums::Document::Document(document) => document,
        _ => {
            println!("upstream document isn't document {:?}", doc);
            return Response::builder().status(500).body(Body::from("failed to call upstream api")).unwrap();
        }
    };

    let file_ref = FileRefV1 {
        message_id: res.id,
        document_id: doc.id,
        file_reference: doc.file_reference.clone(),
        access_hash: doc.access_hash,
        file_size: doc.size,
    };

    let file_ref = FileRef {
        v1: Some(file_ref),
    };

    let file_ref = file_ref.encode_to_vec();
    let file_ref = base64::prelude::BASE64_URL_SAFE_NO_PAD.encode(file_ref);

    let res = UploadFinalizeResponse {
        r#ref: file_ref,
    };
    
    Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_vec(&res).unwrap()))
    .unwrap()
}