use base64::Engine;
use prost::Message;

use crate::shared::CHUNK_SIZE;

include!(concat!(env!("OUT_DIR"), "/_.rs"));

impl FileRef {
    pub fn to_ref_string(&self) -> String {
        base64::prelude::BASE64_URL_SAFE_NO_PAD.encode(self.encode_to_vec())
    }

    pub fn from_ref_string(input: String) -> Option<FileRef> {
        let decoded = base64::prelude::BASE64_URL_SAFE_NO_PAD.decode(input);
        let decoded = match decoded {
            Err(_) => return None,
            Ok(v) => v,
        };
        let decoded = FileRef::decode(&decoded[..]);
        match decoded {
            Ok(v) => Some(v),
            Err(_) => None,
        }
    }
}

impl UploadToken {
    pub fn from_api_string(input: String) -> Option<UploadToken> {
        let decoded = base64::prelude::BASE64_URL_SAFE_NO_PAD.decode(input);
        let decoded = match decoded {
            Err(_) => return None,
            Ok(v) => v,
        };
        let decoded = UploadToken::decode(&decoded[..]);
        match decoded {
            Ok(v) => Some(v),
            Err(_) => None
        }
    }

    pub fn to_api_string(&self) -> String {
        base64::prelude::BASE64_URL_SAFE_NO_PAD.encode(self.encode_to_vec())
    }
}

impl UploadTokenV1 {
    pub fn should_use_big_upload(&self) -> bool {
        self.file_size >= 10 * 1024 * 1024
    }

    pub fn total_parts(&self) -> i32 {
        ((self.file_size + (CHUNK_SIZE as i64) - 1) / (CHUNK_SIZE as i64)) as i32
    }
}