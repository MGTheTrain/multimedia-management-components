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

use application::mm_service::MultimediaManagementService;
use blob_storage_connector::aws_s3_connector::{AwsS3BucketConfig, AwsS3Connector};
use grpc_handlers::v1::{
    multimedia::multimedia_service_server::MultimediaServiceServer, MultimediaGrpcService,
};
use persistence::psql_repository::{PsqlConfig, PsqlRepository};
use std::sync::Arc;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    dotenvy::dotenv().ok();

    let blob = AwsS3Connector::new(AwsS3BucketConfig {
        bucket_name: std::env::var("AWS_BUCKET_NAME").expect("AWS_BUCKET_NAME not set"),
        endpoint_url: std::env::var("AWS_ENDPOINT_URL").expect("AWS_ENDPOINT_URL not set"),
        region: std::env::var("AWS_DEFAULT_REGION").expect("AWS_DEFAULT_REGION not set"),
        access_key_id: std::env::var("AWS_ACCESS_KEY_ID").expect("AWS_ACCESS_KEY_ID not set"),
        secret_access_key: std::env::var("AWS_SECRET_ACCESS_KEY")
            .expect("AWS_SECRET_ACCESS_KEY not set"),
    })
    .await;
    blob.create_bucket().await.ok();

    let repo = PsqlRepository::new(PsqlConfig {
        database_url: std::env::var("DATABASE_URL").expect("DATABASE_URL not set"),
    })
    .await
    .expect("Failed to connect");

    let service = Arc::new(MultimediaManagementService::new(
        Box::new(blob),
        Box::new(repo.clone()),
        Box::new(repo.clone()),
        Box::new(repo.clone()),
        Box::new(repo),
    ));

    let addr = "0.0.0.0:50051".parse()?;
    log::info!("gRPC server listening on {}", addr);

    Server::builder()
        .add_service(
            MultimediaServiceServer::new(MultimediaGrpcService { service })
                .max_decoding_message_size(100 * 1024 * 1024),
        )
        .serve(addr)
        .await?;

    Ok(())
}
