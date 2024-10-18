use grammers_client::{Client, grammers_tl_types as tl};

use crate::{proto::FileRefV1, shared::message_to_file_ref};

pub mod chunk;
pub mod meta;

pub async fn refresh_file_reference(client: &Client, file_ref: &FileRefV1) -> Option<String> {
    let req = tl::functions::messages::GetMessages {
        id: vec![tl::enums::InputMessage::Id(tl::types::InputMessageId {
            id: file_ref.message_id,
        })]
    };
    let res = client.invoke(&req).await;

    let res = match res {
        Err(e) => {
            println!("failed to get message {:?}", e);
            return None;
        }
        Ok(v) => v,
    };

    let res = match res {
        tl::enums::messages::Messages::Messages(m) => m,
        _ => {
            println!("not expected messages {:?}", res);
            return None;
        }
    };

    let res = &res.messages[0];

    let res = match res {
        tl::enums::Message::Empty(message_empty) => {
            println!("message not found {:?}", message_empty);
            return None;
        },
        tl::enums::Message::Message(message) => message,
        tl::enums::Message::Service(message_service) => todo!(),
    };

    let file_ref = message_to_file_ref(res);

    return file_ref.map(|x| x.to_ref_string());
}
