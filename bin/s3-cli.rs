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

use clap::{Parser, Subcommand};
use connectors::aws_s3_connector::{AwsS3BucketConfig, AwsS3Connector};
use connectors::blob_storage_connector::BlobStorageConnector;
use log::info;

#[derive(Parser, Debug)]
#[clap(
    version = "1.0.0",
    about = "CLI for blob and bucket operations on AWS S3 or S3-compatible storage"
)]
struct Cli {
    #[clap(subcommand)]
    operation: S3Operation,
}

#[derive(Debug, Subcommand)]
enum S3Operation {
    /// Create bucket
    CreateBucket,
    /// Upload a blob from a file
    UploadBlob {
        #[clap(short, long)]
        blob_name: String,
        #[clap(short, long)]
        file_path: String,
    },
    /// Download a blob to a file
    DownloadBlob {
        #[clap(short, long)]
        blob_name: String,
        #[clap(short, long)]
        output_path: String,
    },
    /// Delete a blob
    DeleteBlob {
        #[clap(short, long)]
        blob_name: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    dotenvy::dotenv().ok();

    let connector = AwsS3Connector::new(AwsS3BucketConfig {
        bucket_name: std::env::var("AWS_BUCKET_NAME").expect("AWS_BUCKET_NAME not set"),
        endpoint_url: std::env::var("AWS_ENDPOINT_URL").unwrap_or_default(),
        region: std::env::var("AWS_DEFAULT_REGION").expect("AWS_DEFAULT_REGION not set"),
        access_key_id: std::env::var("AWS_ACCESS_KEY_ID").expect("AWS_ACCESS_KEY_ID not set"),
        secret_access_key: std::env::var("AWS_SECRET_ACCESS_KEY")
            .expect("AWS_SECRET_ACCESS_KEY not set"),
    })
    .await;

    let args = Cli::parse();

    match args.operation {
        S3Operation::CreateBucket => {
            connector.create_bucket().await?;
            info!("Bucket created");
        }
        S3Operation::UploadBlob {
            blob_name,
            file_path,
        } => {
            connector
                .upload_blob_bytes(&blob_name, &std::fs::read(&file_path)?)
                .await?;
            info!("Uploaded {} as {}", file_path, blob_name);
        }
        S3Operation::DownloadBlob {
            blob_name,
            output_path,
        } => {
            let bytes = connector.download_blob(&blob_name).await?;
            std::fs::write(&output_path, bytes)?;
            info!("Downloaded {} to {}", blob_name, output_path);
        }
        S3Operation::DeleteBlob { blob_name } => {
            connector.delete_blob(&blob_name).await?;
            info!("Deleted {}", blob_name);
        }
    }

    Ok(())
}
