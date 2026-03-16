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

use crate::mm_service_trait::MultimediaManagementServiceTrait;
use async_trait::async_trait;
use chrono::Utc;
use connectors::blob_storage_connector::BlobStorageConnector;
use data_access::psql_data_access::{PsqlDataAccess, PsqlDataAccessConfig};
use log::info;
use models::{
    container_meta::ContainerMeta,
    tracks::{AudioTrack, SubtitleTrack, VideoTrack},
};
use parsers::mp4_parser::Mp4Parser;
use uuid::Uuid;

#[derive(Clone)]
pub struct MultimediaManagementService<C: BlobStorageConnector + Clone + Send + Sync> {
    connector: C,
    db: PsqlDataAccess,
}

impl<C: BlobStorageConnector + Clone + Send + Sync> MultimediaManagementService<C> {
    pub async fn new(
        connector: C,
        db_config: PsqlDataAccessConfig,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(MultimediaManagementService {
            connector,
            db: PsqlDataAccess::new(db_config).await?,
        })
    }
}

#[async_trait]
impl<C: BlobStorageConnector + Clone + Send + Sync> MultimediaManagementServiceTrait
    for MultimediaManagementService<C>
{
    async fn upload(
        &self,
        blob_name: &str,
        file_path: &str,
        tags: Vec<Option<String>>,
    ) -> Result<ContainerMeta, Box<dyn std::error::Error + Send + Sync>> {
        let container_meta_id = Uuid::new_v4();
        let full_blob_name = format!("{}/{}", container_meta_id, blob_name);

        self.connector
            .upload_blob(&full_blob_name, file_path)
            .await?;

        let result = Mp4Parser::parse_from_file(file_path)?;
        let mut container_meta = result.container;
        let video_track = result.video;
        let audio_track = result.audio;
        let subtitle_track = result.subtitle;

        if let Some(mut t) = video_track {
            t.id = Uuid::new_v4();
            t.container_meta_id = container_meta_id;
            self.db.insert_video_track(&t).await?;
            container_meta.video_track_id = Some(t.id);
        }
        if let Some(mut t) = audio_track {
            t.id = Uuid::new_v4();
            t.container_meta_id = container_meta_id;
            self.db.insert_audio_track(&t).await?;
            container_meta.audio_track_id = Some(t.id);
        }
        if let Some(mut t) = subtitle_track {
            t.id = Uuid::new_v4();
            t.container_meta_id = container_meta_id;
            self.db.insert_subtitle_track(&t).await?;
            container_meta.subtitle_track_id = Some(t.id);
        }

        let now = Utc::now();
        container_meta.title = blob_name.to_string();
        container_meta.id = container_meta_id;
        container_meta.date_time_created = now;
        container_meta.date_time_updated = now;
        container_meta.tags = tags;

        self.db.insert_container_meta(&container_meta).await?;
        info!("Uploaded and created metadata for {}", container_meta_id);
        Ok(container_meta)
    }

    async fn upload_bytes(
        &self,
        blob_name: &str,
        data: &[u8],
        tags: Vec<Option<String>>,
    ) -> Result<ContainerMeta, Box<dyn std::error::Error + Send + Sync>> {
        let container_meta_id = Uuid::new_v4();
        let full_blob_name = format!("{}/{}", container_meta_id, blob_name);

        // write temp file only for parser
        let tmp_path = format!("/tmp/{}", blob_name);
        std::fs::write(&tmp_path, data)?;

        self.connector
            .upload_blob_bytes(&full_blob_name, data)
            .await?;

        let result = Mp4Parser::parse_from_file(&tmp_path)?;
        let mut container_meta = result.container;

        if let Some(mut t) = result.video {
            t.id = Uuid::new_v4();
            t.container_meta_id = container_meta_id;
            self.db.insert_video_track(&t).await?;
            container_meta.video_track_id = Some(t.id);
        }
        if let Some(mut t) = result.audio {
            t.id = Uuid::new_v4();
            t.container_meta_id = container_meta_id;
            self.db.insert_audio_track(&t).await?;
            container_meta.audio_track_id = Some(t.id);
        }
        if let Some(mut t) = result.subtitle {
            t.id = Uuid::new_v4();
            t.container_meta_id = container_meta_id;
            self.db.insert_subtitle_track(&t).await?;
            container_meta.subtitle_track_id = Some(t.id);
        }

        let now = Utc::now();
        container_meta.title = blob_name.to_string();
        container_meta.id = container_meta_id;
        container_meta.date_time_created = now;
        container_meta.date_time_updated = now;
        container_meta.tags = tags;

        self.db.insert_container_meta(&container_meta).await?;
        info!(
            "Uploaded bytes and created metadata for {}",
            container_meta_id
        );
        Ok(container_meta)
    }

    async fn download(
        &self,
        container_meta_id: &str,
        blob_name: &str,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let full_blob_name = format!("{}/{}", container_meta_id, blob_name);
        let bytes = self.connector.download_blob(&full_blob_name).await?;
        info!("Downloaded blob {}", full_blob_name);
        Ok(bytes)
    }

    async fn get_container_meta(
        &self,
        id: &Uuid,
    ) -> Result<ContainerMeta, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.db.get_container_meta_by_id(id).await?)
    }

    async fn get_video_track(
        &self,
        id: &Uuid,
    ) -> Result<VideoTrack, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.db.get_video_track_by_id(id).await?)
    }

    async fn get_audio_track(
        &self,
        id: &Uuid,
    ) -> Result<AudioTrack, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.db.get_audio_track_by_id(id).await?)
    }

    async fn get_subtitle_track(
        &self,
        id: &Uuid,
    ) -> Result<SubtitleTrack, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.db.get_subtitle_track_by_id(id).await?)
    }

    async fn delete(
        &self,
        container_meta_id: &str,
        blob_name: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let full_blob_name = format!("{}/{}", container_meta_id, blob_name);
        self.connector.delete_blob(&full_blob_name).await?;

        let id = Uuid::parse_str(container_meta_id)?;
        self.db.delete_container_meta_by_id(&id).await?;
        info!("Deleted blob and metadata for {}", id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use connectors::aws_s3_connector::{AwsS3BucketConfig, AwsS3Connector};
    use data_access::psql_data_access::PsqlDataAccessConfig;

    #[tokio::test]
    async fn test_multimedia_management_service() -> Result<(), Box<dyn std::error::Error>> {
        let _ = env_logger::try_init();

        let connector = AwsS3Connector::new(AwsS3BucketConfig {
            bucket_name: String::from("test-bucket"),
            endpoint_url: String::from("http://127.0.0.1:4566"),
            region: String::from("us-east-1"),
            access_key_id: String::from("test"),
            secret_access_key: String::from("test"),
        })
        .await;
        connector.create_bucket().await?;

        let service = MultimediaManagementService::new(
            connector,
            PsqlDataAccessConfig {
                database_url: String::from("postgres://user:password@localhost:5432/diesel-demo"),
            },
        )
        .await?;

        let blob_name = "nature.mp4";
        let file_path = "../assets/nature.mp4";
        let tags = vec![
            Some(String::from("nature")),
            Some(String::from("adventure")),
        ];

        // [C]reate
        let meta = service
            .upload(blob_name, file_path, tags)
            .await
            .map_err(|e| format!("upload failed: {}", e))?;

        // [R]ead
        let container_meta = service
            .get_container_meta(&meta.id)
            .await
            .map_err(|e| format!("get_container_meta failed: {}", e))?;
        assert_eq!(container_meta.id, meta.id);

        let video_track = service
            .get_video_track(&meta.video_track_id.unwrap())
            .await
            .map_err(|e| format!("get_video_track failed: {}", e))?;
        assert_eq!(video_track.container_meta_id, meta.id);
        assert_eq!(video_track.media_type, "h264");

        let audio_track = service
            .get_audio_track(&meta.audio_track_id.unwrap())
            .await
            .map_err(|e| format!("get_audio_track failed: {}", e))?;
        assert_eq!(audio_track.container_meta_id, meta.id);
        assert_eq!(audio_track.media_type, "aac");

        // [D]ownload
        let bytes = service
            .download(&meta.id.to_string(), blob_name)
            .await
            .map_err(|e| format!("download failed: {}", e))?;
        assert!(!bytes.is_empty());

        // [D]elete
        service
            .delete(&meta.id.to_string(), blob_name)
            .await
            .map_err(|e| format!("delete failed: {}", e))?;

        Ok(())
    }

    #[tokio::test]
    async fn test_multimedia_management_service_upload_bytes(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let _ = env_logger::try_init();

        let connector = AwsS3Connector::new(AwsS3BucketConfig {
            bucket_name: String::from("test-bucket"),
            endpoint_url: String::from("http://127.0.0.1:4566"),
            region: String::from("us-east-1"),
            access_key_id: String::from("test"),
            secret_access_key: String::from("test"),
        })
        .await;
        connector.create_bucket().await?;

        let service = MultimediaManagementService::new(
            connector,
            PsqlDataAccessConfig {
                database_url: String::from("postgres://user:password@localhost:5432/diesel-demo"),
            },
        )
        .await?;

        let blob_name = "nature.mp4";
        let data = std::fs::read("../assets/nature.mp4")
            .map_err(|e| format!("failed to read asset: {}", e))?;
        let tags = vec![
            Some(String::from("nature")),
            Some(String::from("adventure")),
        ];

        // [C]reate via bytes
        let meta = service
            .upload_bytes(blob_name, &data, tags)
            .await
            .map_err(|e| format!("upload_bytes failed: {}", e))?;

        // [R]ead
        let container_meta = service
            .get_container_meta(&meta.id)
            .await
            .map_err(|e| format!("get_container_meta failed: {}", e))?;
        assert_eq!(container_meta.id, meta.id);

        let video_track = service
            .get_video_track(&meta.video_track_id.unwrap())
            .await
            .map_err(|e| format!("get_video_track failed: {}", e))?;
        assert_eq!(video_track.container_meta_id, meta.id);
        assert_eq!(video_track.media_type, "h264");

        let audio_track = service
            .get_audio_track(&meta.audio_track_id.unwrap())
            .await
            .map_err(|e| format!("get_audio_track failed: {}", e))?;
        assert_eq!(audio_track.container_meta_id, meta.id);
        assert_eq!(audio_track.media_type, "aac");

        // [D]ownload
        let bytes = service
            .download(&meta.id.to_string(), blob_name)
            .await
            .map_err(|e| format!("download failed: {}", e))?;
        assert!(!bytes.is_empty());

        // [D]elete
        service
            .delete(&meta.id.to_string(), blob_name)
            .await
            .map_err(|e| format!("delete failed: {}", e))?;

        Ok(())
    }
}
