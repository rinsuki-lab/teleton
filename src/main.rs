use axum::{body::Bytes, extract::{Path, Query}, http::HeaderMap, routing::{get, post}, Json};

mod teleauth;
mod handlers;
pub mod shared;

pub mod proto;

#[tokio::main]
async fn main() {
    let client = teleauth::get_authorized_client().await;

    println!("Starting...");

    let app = axum::Router::new();
    let app = app.route("/", get(|| async { "Hello, world!" }));
    let app = {
        let client = client.clone();
        app.route("/v1/upload/limit", get(|| async move {
            handlers::upload::get_upload_limit(&client).await
        }))
    };
    let app = {
        app.route("/v1/upload/start", post(|Query(query): Query<handlers::upload::StartUploadQueryParams>| async move {
            handlers::upload::start_upload(query).await
        }))
    };
    let app = {
        let client = client.clone();
        app.route("/v1/upload/chunk", post(|Query(query): Query<handlers::upload::UploadChunkQueryParams>, body: Bytes| async move {
            handlers::upload::upload_chunk(&client, query, Vec::from(body)).await
        }))
    };
    let app = {
        let client = client.clone();
        app.route("/v1/upload/finalize", post(|Query(query): Query<handlers::upload::UploadFinalizeQueryParams>, Json(body): Json<handlers::upload::UploadFinalizeBody>| async move {
            handlers::upload::upload_finalize(query, body, &client).await
        }))
    };
    let app = {
        let client = client.clone();
        app.route("/v1/files/:file_ref/chunks/:offset", get(|Path((file_ref, offset)): Path<(String, usize)>, headers: HeaderMap| async move {
            handlers::files::chunk::get_chunk(&client, file_ref, offset).await
        }))
    };
    let app = {
        let client = client.clone();
        app.route("/v1/files/:file_ref/meta", get(|Path(file_ref): Path<String>, headers: HeaderMap| async move {
            handlers::files::meta::get_file_meta(&client, file_ref).await
        }))
    };

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.expect("Failed to bind");
    axum::serve(listener, app).await.unwrap();
}
