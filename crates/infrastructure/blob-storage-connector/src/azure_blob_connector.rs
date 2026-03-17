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

use async_trait::async_trait;
use azure_storage::prelude::*;
use azure_storage_blobs::prelude::*;
use domain::blob_storage_connector::{BlobStorageConnector, BlobStorageError};
use log::info;

pub struct AzureBlobStorageAccountConfig {
    pub account_name: String,
    pub access_key: String,
    pub container_name: String,
    pub endpoint_url: String,
}

#[derive(Clone)]
pub struct AzureBlobConnector {
    container_client: ContainerClient,
}

// inherent methods — not part of the trait
impl AzureBlobConnector {
    pub fn new(config: AzureBlobStorageAccountConfig) -> Self {
        let credentials =
            StorageCredentials::access_key(config.account_name.clone(), config.access_key);
        let container_client = if config.endpoint_url.contains("127.0.0.1")
            || config.endpoint_url.contains("localhost")
        {
            ClientBuilder::emulator().container_client(config.container_name)
        } else {
            ClientBuilder::new(config.account_name, credentials)
                .container_client(config.container_name)
        };
        AzureBlobConnector { container_client }
    }

    pub async fn create_container(&self) -> Result<(), Box<dyn std::error::Error>> {
        match self.container_client.create().await {
            Ok(_) => info!("Created container"),
            Err(e) => {
                let err_str = e.to_string();
                if err_str.contains("409") || err_str.contains("ContainerAlreadyExists") {
                    info!("Container already exists, skipping creation");
                } else {
                    return Err(e.into());
                }
            }
        }
        Ok(())
    }
}

// trait impl — separate block
#[async_trait]
impl BlobStorageConnector for AzureBlobConnector {
    async fn upload_bytes(&self, blob_name: &str, data: &[u8]) -> Result<(), BlobStorageError> {
        self.container_client
            .blob_client(blob_name)
            .put_block_blob(data.to_vec())
            .await
            .map_err(|e| BlobStorageError::Storage(e.to_string()))?;
        info!("Uploaded blob bytes {}", blob_name);
        Ok(())
    }

    async fn download(&self, blob_name: &str) -> Result<Vec<u8>, BlobStorageError> {
        let data = self
            .container_client
            .blob_client(blob_name)
            .get_content()
            .await
            .map_err(|e| BlobStorageError::Storage(e.to_string()))?;
        info!("Downloaded blob {}", blob_name);
        Ok(data)
    }

    async fn delete(&self, blob_name: &str) -> Result<(), BlobStorageError> {
        self.container_client
            .blob_client(blob_name)
            .delete()
            .await
            .map_err(|e| BlobStorageError::Storage(e.to_string()))?;
        info!("Deleted blob {}", blob_name);
        Ok(())
    }
}

#[tokio::test]
async fn test_azure_blob_connector() -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Write;
    use tempfile::NamedTempFile;

    let config = AzureBlobStorageAccountConfig {
        account_name: String::from("devstoreaccount1"),
        access_key: String::from("Eby8vdM02xNOcqFlqUwJPLlmEtlCDXJ1OUzFT50uSRZ6IFsuFq2UVErCz4I6tq/K1SZFPTOtr/KBHBeksoGMGw=="),
        container_name: String::from("test-container"),
        endpoint_url: String::from("http://127.0.0.1:10000"),
    };
    let connector = AzureBlobConnector::new(config);
    connector.create_container().await?;

    let mut tmp = NamedTempFile::new()?;
    writeln!(tmp, "sample content")?;

    // upload from bytes
    let blob_name_bytes = format!("{}/sample_bytes.txt", uuid::Uuid::new_v4());
    let upload_bytes = connector
        .upload_bytes(&blob_name_bytes, b"sample content\n")
        .await;
    assert!(upload_bytes.is_ok());

    let bytes = connector.download(&blob_name_bytes).await?;
    assert!(!bytes.is_empty());
    assert_eq!(bytes, b"sample content\n");

    connector.delete(&blob_name_bytes).await?;

    Ok(())
}
