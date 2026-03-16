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

use crate::blob_storage_connector::{BlobStorageConnector, BlobStorageError};
use async_trait::async_trait;
use aws_config::BehaviorVersion;
use aws_sdk_s3::{primitives::ByteStream, Client, Error};
use log::info;
use std::path::Path;

pub struct AwsS3BucketConfig {
    pub bucket_name: String,
    pub endpoint_url: String,
    pub region: String,
    pub access_key_id: String,
    pub secret_access_key: String,
}

#[derive(Clone)]
pub struct AwsS3Connector {
    bucket_name: String,
    client: Client,
}

// inherent methods — not part of the trait
impl AwsS3Connector {
    pub async fn new(config: AwsS3BucketConfig) -> Self {
        std::env::set_var("AWS_ACCESS_KEY_ID", &config.access_key_id);
        std::env::set_var("AWS_SECRET_ACCESS_KEY", &config.secret_access_key);
        std::env::set_var("AWS_DEFAULT_REGION", &config.region);
        std::env::set_var("AWS_ENDPOINT_URL", &config.endpoint_url);
        let aws_config = aws_config::load_defaults(BehaviorVersion::latest()).await;
        AwsS3Connector {
            bucket_name: config.bucket_name,
            client: Client::from_conf(
                aws_sdk_s3::config::Builder::from(&aws_config)
                    .force_path_style(true)
                    .build(),
            ),
        }
    }

    pub async fn create_bucket(&self) -> Result<(), Error> {
        self.client
            .create_bucket()
            .bucket(&self.bucket_name)
            .send()
            .await?;
        info!("Created bucket {}", self.bucket_name);
        Ok(())
    }
}

// trait impl — separate block
#[async_trait]
impl BlobStorageConnector for AwsS3Connector {
    async fn upload_blob(&self, blob_name: &str, file_path: &str) -> Result<(), BlobStorageError> {
        let body = ByteStream::from_path(Path::new(file_path))
            .await
            .map_err(|e| BlobStorageError::Storage(e.to_string()))?;
        self.client
            .put_object()
            .bucket(&self.bucket_name)
            .key(blob_name)
            .body(body)
            .send()
            .await
            .map_err(|e| BlobStorageError::Storage(e.to_string()))?;
        info!("Uploaded blob {}", blob_name);
        Ok(())
    }

    async fn upload_blob_bytes(
        &self,
        blob_name: &str,
        data: &[u8],
    ) -> Result<(), BlobStorageError> {
        let body = ByteStream::from(data.to_vec());
        self.client
            .put_object()
            .bucket(&self.bucket_name)
            .key(blob_name)
            .body(body)
            .send()
            .await
            .map_err(|e| BlobStorageError::Storage(e.to_string()))?;
        info!("Uploaded blob bytes {}", blob_name);
        Ok(())
    }

    async fn download_blob(&self, blob_name: &str) -> Result<Vec<u8>, BlobStorageError> {
        let output = self
            .client
            .get_object()
            .bucket(&self.bucket_name)
            .key(blob_name)
            .send()
            .await
            .map_err(|e| BlobStorageError::Storage(e.to_string()))?;
        let bytes = output
            .body
            .collect()
            .await
            .map_err(|e| BlobStorageError::Storage(e.to_string()))?
            .into_bytes()
            .to_vec();
        info!("Downloaded blob {}", blob_name);
        Ok(bytes)
    }

    async fn delete_blob(&self, blob_name: &str) -> Result<(), BlobStorageError> {
        self.client
            .delete_object()
            .bucket(&self.bucket_name)
            .key(blob_name)
            .send()
            .await
            .map_err(|e| BlobStorageError::Storage(e.to_string()))?;
        info!("Deleted blob {}", blob_name);
        Ok(())
    }
}

#[tokio::test]
async fn test_aws_s3_connector() -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Write;
    use tempfile::NamedTempFile;

    let config = AwsS3BucketConfig {
        bucket_name: String::from("test-bucket"),
        endpoint_url: String::from("http://127.0.0.1:4566"),
        region: String::from("us-east-1"),
        access_key_id: String::from("test"),
        secret_access_key: String::from("test"),
    };
    let connector = AwsS3Connector::new(config).await;
    connector.create_bucket().await?;

    let mut tmp = NamedTempFile::new()?;
    writeln!(tmp, "sample content")?;
    let blob_name = format!("{}/sample.txt", uuid::Uuid::new_v4());

    // upload from file path
    let upload = connector
        .upload_blob(&blob_name, tmp.path().to_str().unwrap())
        .await;
    assert!(upload.is_ok());

    let bytes = connector.download_blob(&blob_name).await?;
    assert!(!bytes.is_empty());
    assert_eq!(bytes, b"sample content\n");

    connector.delete_blob(&blob_name).await?;

    // upload from bytes
    let blob_name_bytes = format!("{}/sample_bytes.txt", uuid::Uuid::new_v4());
    let upload_bytes = connector
        .upload_blob_bytes(&blob_name_bytes, b"sample content\n")
        .await;
    assert!(upload_bytes.is_ok());

    let bytes = connector.download_blob(&blob_name_bytes).await?;
    assert!(!bytes.is_empty());
    assert_eq!(bytes, b"sample content\n");

    connector.delete_blob(&blob_name_bytes).await?;

    Ok(())
}
