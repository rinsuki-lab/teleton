use grammers_client::grammers_tl_types as tl;

use crate::proto::{FileRef, FileRefV1};

pub const CHUNK_SIZE: usize = 512 * 1024;

pub fn message_to_file_ref(message: &tl::types::Message) -> Option<FileRef> {
    let doc = match &message.media {
        None => {
            println!("upstream doesn't contains media in message {:?}", message);
            return None;
        },
        Some(v) => v,
    };
    let doc = match doc {
        tl::enums::MessageMedia::Document(d) => d,
        _ => {
            println!("upstream media isn't document {:?}", doc);
            return None;
        }
    };
    let doc = match &doc.document {
        Some(v) => v,
        None => {
            println!("upstream document isn't available {:?}", doc);
            return None;
        },
    };
    let doc = match doc {
        tl::enums::Document::Document(document) => document,
        _ => {
            println!("upstream document isn't document {:?}", doc);
            return None;
        }
    };

    let file_ref = FileRefV1 {
        message_id: message.id,
        document_id: doc.id,
        file_reference: doc.file_reference.clone(),
        access_hash: doc.access_hash,
        file_size: doc.size,
    };

    let file_ref = FileRef {
        v1: Some(file_ref),
    };

    return Some(file_ref);
}