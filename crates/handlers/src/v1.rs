// The MIT License
//
// Copyright (c) 2026 MGTheTrain
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use application::mm_service_trait::MultimediaManagementServiceTrait;
use axum::extract::Query;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use std::collections::HashMap;
use std::sync::Arc;

pub type AppState = Arc<dyn MultimediaManagementServiceTrait + Send + Sync>;

pub async fn upload(
    State(service): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    body: axum::body::Bytes,
) -> impl IntoResponse {
    let file_name = params
        .get("filename")
        .map(|s| s.as_str())
        .unwrap_or("upload")
        .to_string();
    let tags = vec![Some(String::from("uploaded"))];
    match service.upload_bytes(&file_name, &body, tags).await {
        Ok(meta) => (
            StatusCode::CREATED,
            Json(serde_json::json!({ "id": meta.id })),
        )
            .into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn download(
    State(service): State<AppState>,
    axum::extract::Path((container_meta_id, blob_name)): axum::extract::Path<(String, String)>,
) -> impl IntoResponse {
    match service.download(&container_meta_id, &blob_name).await {
        Ok(bytes) => (StatusCode::OK, bytes).into_response(),
        Err(e) => (StatusCode::NOT_FOUND, e.to_string()).into_response(),
    }
}

pub async fn delete(
    State(service): State<AppState>,
    axum::extract::Path((container_meta_id, blob_name)): axum::extract::Path<(String, String)>,
) -> impl IntoResponse {
    match service.delete(&container_meta_id, &blob_name).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

// TODO: fix e2e-test
// #[cfg(test)]
// mod tests {
//     use super::*;
//     use axum::routing::{post, get, delete as axum_delete};
//     use axum::Router;
//     use axum_test::{TestServer, TestServerConfig, Transport};
//     use axum::http::StatusCode;
//     use connectors::aws_s3_connector::{AwsS3BucketConfig, AwsS3Connector};
//     use data_access::psql_data_access::PsqlDataAccessConfig;
//     use services::mm_service::MultimediaManagementService;
//     use std::sync::Arc;

//     async fn build_test_server() -> TestServer {
//         let connector = AwsS3Connector::new(AwsS3BucketConfig {
//             bucket_name: String::from("test-bucket"),
//             endpoint_url: String::from("http://127.0.0.1:4566"),
//             region: String::from("us-east-1"),
//             access_key_id: String::from("test"),
//             secret_access_key: String::from("test"),
//         }).await;
//         connector.create_bucket().await.unwrap();

//         let service = MultimediaManagementService::new(
//             connector,
//             PsqlDataAccessConfig {
//                 database_url: String::from("postgres://user:password@localhost:5432/diesel-demo"),
//             },
//         ).await.unwrap();

//         let state: AppState = Arc::new(service);

//         let app = Router::new()
//             .route("/blobs", post(upload))
//             .route("/blobs/{container_meta_id}/{blob_name}", get(download))
//             .route("/blobs/{container_meta_id}/{blob_name}", axum_delete(delete))
//             .with_state(state);

//         let config = TestServerConfig {
//             transport: Some(Transport::HttpRandomPort),
//             ..Default::default()
//         };
//         TestServer::new_with_config(app, config)
//     }

//     #[tokio::test]
//     async fn test_upload_download_delete() {
//         let server = build_test_server().await;

//         let mp4_bytes = std::fs::read(
//             concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/nature.mp4")
//         ).unwrap();

//         let response = server
//             .post("/blobs?filename=nature.mp4")
//             .bytes(mp4_bytes.into())
//             .content_type("video/mp4")
//             .await;
//         response.assert_status(StatusCode::CREATED);

//         let body: serde_json::Value = response.json();
//         let container_meta_id = body["id"].as_str().unwrap().to_string();

//         let response = server.get(&format!("/blobs/{}/nature.mp4", container_meta_id)).await;
//         response.assert_status(StatusCode::OK);
//         assert!(!response.as_bytes().is_empty());

//         let response = server.delete(&format!("/blobs/{}/nature.mp4", container_meta_id)).await;
//         response.assert_status(StatusCode::NO_CONTENT);
//     }
// }
