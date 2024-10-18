use axum::{body::Body, response::Response};
use grammers_client::{grammers_tl_types as tl, Client};

use crate::shared::CHUNK_SIZE;


#[derive(serde::Serialize)]
struct UploadLimitResponse {
    file_size_limit: usize,
}

pub async fn get_upload_limit(client: &Client) -> Response {
    let user = tl::functions::users::GetUsers {
        id: vec![tl::enums::InputUser::UserSelf]
    };

    let user = match client.invoke(&user).await {
        Ok(v) => v,
        Err(e) => {
            println!("Failed to get current my status {:?}", e);
            return Response::builder().status(500).body(Body::from("Failed to call upstream user API")).unwrap();
        }
    };

    let user = &user[0];

    let user = match user {
        tl::enums::User::Empty(user_empty) => todo!(),
        tl::enums::User::User(user) => user,
    };

    let config = tl::functions::help::GetAppConfig {
        hash: 0,
    };

    let config = match client.invoke(&config).await {
        Ok(v) => v,
        Err(e) => {
            println!("Failed to get config {:?}", e);
            return Response::builder().status(500).body(Body::from("Failed to call upstream config API")).unwrap();
        }
    };

    let config = match config {
        tl::enums::help::AppConfig::NotModified => todo!(),
        tl::enums::help::AppConfig::Config(app_config) => app_config,
    };

    let config: tl::types::JsonObject = match config.config {
        tl::enums::Jsonvalue::JsonNull => todo!(),
        tl::enums::Jsonvalue::JsonBool(json_bool) => todo!(),
        tl::enums::Jsonvalue::JsonNumber(json_number) => todo!(),
        tl::enums::Jsonvalue::JsonString(json_string) => todo!(),
        tl::enums::Jsonvalue::JsonArray(json_array) => todo!(),
        tl::enums::Jsonvalue::JsonObject(json_object) => json_object,
    };

    let max_chunk_count_key = if user.premium { "upload_max_fileparts_premium" } else { "upload_max_fileparts_default" };

    let max_chunk_count = config.value.iter().map(|x| match x {
        tl::enums::JsonobjectValue::JsonObjectValue(json_object_value) => json_object_value,
    }).find(|x| { x.key == max_chunk_count_key });

    let max_chunk_count = match max_chunk_count {
        Some(v) => match &v.value {
            tl::enums::Jsonvalue::JsonNull => todo!(),
            tl::enums::Jsonvalue::JsonBool(json_bool) => todo!(),
            tl::enums::Jsonvalue::JsonNumber(json_number) => json_number,
            tl::enums::Jsonvalue::JsonString(json_string) => todo!(),
            tl::enums::Jsonvalue::JsonArray(json_array) => todo!(),
            tl::enums::Jsonvalue::JsonObject(json_object) => todo!(),
        },
        None => {
            return Response::builder().status(500).body(Body::from("Failed to get max chunk count from upstream API")).unwrap();
        },
    };

    let res = UploadLimitResponse {
        file_size_limit: max_chunk_count.value as usize * CHUNK_SIZE,
    };
    let res = serde_json::to_vec(&res).unwrap();

    Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .body(Body::from(res))
        .unwrap()
}