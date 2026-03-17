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

use blob_storage_connector::azure_blob_connector::{
    AzureBlobConnector, AzureBlobStorageAccountConfig,
};
use clap::{Parser, Subcommand};
use domain::blob_storage_connector::BlobStorageConnector;
use log::info;

#[derive(Parser, Debug)]
#[clap(
    version = "1.0.0",
    about = "CLI for blob and container operations on Azure Blob Storage"
)]
struct Cli {
    #[clap(subcommand)]
    operation: AzureOperation,
}

#[derive(Debug, Subcommand)]
enum AzureOperation {
    /// Create container
    CreateContainer,
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

    let connector = AzureBlobConnector::new(AzureBlobStorageAccountConfig {
        account_name: std::env::var("AZURE_STORAGE_ACCOUNT")
            .expect("AZURE_STORAGE_ACCOUNT not set"),
        access_key: std::env::var("AZURE_STORAGE_KEY").expect("AZURE_STORAGE_KEY not set"),
        container_name: std::env::var("AZURE_CONTAINER_NAME")
            .expect("AZURE_CONTAINER_NAME not set"),
        endpoint_url: std::env::var("AZURE_ENDPOINT_URL").unwrap_or_default(),
    });

    let args = Cli::parse();

    match args.operation {
        AzureOperation::CreateContainer => {
            connector.create_container().await?;
            info!("Container created");
        }
        AzureOperation::UploadBlob {
            blob_name,
            file_path,
        } => {
            connector
                .upload_bytes(&blob_name, &std::fs::read(&file_path)?)
                .await?;
            info!("Uploaded {} as {}", file_path, blob_name);
        }
        AzureOperation::DownloadBlob {
            blob_name,
            output_path,
        } => {
            let bytes = connector.download(&blob_name).await?;
            std::fs::write(&output_path, bytes)?;
            info!("Downloaded {} to {}", blob_name, output_path);
        }
        AzureOperation::DeleteBlob { blob_name } => {
            connector.delete(&blob_name).await?;
            info!("Deleted {}", blob_name);
        }
    }

    Ok(())
}
