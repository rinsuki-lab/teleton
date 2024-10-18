use axum::{body::Body, response::Response};
use grammers_client::{Client, grammers_tl_types as tl};

use crate::{proto::FileRef, shared::CHUNK_SIZE};

const BYTES_PREFIX: &str = "bytes=";

fn parse_range_header(range_header: String) -> (usize, usize) {
    if !range_header.starts_with(BYTES_PREFIX) {
        return (0, 0);
    }

    let split_pos = range_header.find("-");
    let split_pos = match split_pos {
        None => return (0, 0),
        Some(v) => v,
    };

    if split_pos <= BYTES_PREFIX.len() {
        return (0, 0);
    }

    let start_str = &range_header[BYTES_PREFIX.len()..split_pos];
    let end_str = &range_header[split_pos+1..];

    let start: usize = start_str.parse().unwrap_or(0);
    let end = match end_str.len() {
        0 => 0,
        _ => end_str.parse().unwrap_or(0)
    };

    
    return (start, end)
}

#[cfg(test)]
mod tests {
    use crate::handlers::chunk::parse_range_header;

    #[test]
    fn start_only() {
        assert_eq!(parse_range_header("bytes=0-".to_string()), (0, 0));
        assert_eq!(parse_range_header("bytes=123456-".to_string()), (123456, 0));
    }

    #[test]
    fn start_and_end() {
        assert_eq!(parse_range_header("bytes=0-1".to_string()), (0, 1));
        assert_eq!(parse_range_header("bytes=1234-5678".to_string()), (1234, 5678));
    }
}

pub async fn get_chunk_by_range_header(client: &Client, file_ref: String, range_header: Option<String>) -> Response {
    let range_header = range_header.unwrap_or("bytes=0-".to_string());
    let (start_pos, end_pos) = parse_range_header(range_header);

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

    let req = tl::functions::upload::GetFile {
        cdn_supported: false,
        limit: CHUNK_SIZE as i32,
        location: tl::enums::InputFileLocation::InputDocumentFileLocation(tl::types::InputDocumentFileLocation {
            id: file_ref.document_id, access_hash: file_ref.access_hash, file_reference: file_ref.file_reference, thumb_size: "".to_string()
        }),
        precise: false,
        offset: ((start_pos / CHUNK_SIZE) * CHUNK_SIZE) as i64,
    };

    let res = client.invoke(&req).await;
    let res = match res {
        Ok(v) => v,
        Err(e) => {
            println!("failed to fetch data from upstream {:?}", e);
            return Response::builder().status(500).body(Body::from("failed to fetch from upstream")).unwrap();
        }
    };
    let res = match res {
        tl::enums::upload::File::File(file) => file,
        tl::enums::upload::File::CdnRedirect(file_cdn_redirect) => {
            println!("TODO: support cdn redirect (maybe not though)");
            return Response::builder().status(500).body(Body::from("failed to fetch from upstream")).unwrap();
        },
    };

    let end_pos_max = (req.offset as usize) + res.bytes.len() - 1;
    let end_pos = match end_pos {
        0 => end_pos_max,
        _ => end_pos,
    };
    let end_pos = if end_pos > end_pos_max {
        end_pos_max
    } else {
        end_pos
    };

    let bytes = &res.bytes[..];

    let start_pos_in_bytes = start_pos - (req.offset as usize);
    let mut end_pos_in_bytes = end_pos - (req.offset as usize);

    if end_pos_in_bytes >= (bytes.len() as usize) {
        end_pos_in_bytes = bytes.len();
    }

    let bytes = res.bytes[start_pos_in_bytes..=end_pos_in_bytes].to_vec();
    
    return Response::builder()
        .status(206)
        .header("Content-Type", "video/mp4")
        .header("Content-Range", format!("bytes {}-{}/{}", start_pos, end_pos, file_ref.file_size))
        .body(Body::from(bytes))
    .unwrap();
}