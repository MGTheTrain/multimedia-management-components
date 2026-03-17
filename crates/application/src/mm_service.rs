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
use domain::{
    blob_storage_connector::BlobStorageConnector,
    container_meta::ContainerMeta,
    repositories::{
        AudioTrackRepository, ContainerMetaRepository, SubtitleTrackRepository,
        VideoTrackRepository,
    },
    tracks::{AudioTrack, SubtitleTrack, VideoTrack},
};
use log::info;
use parsers::mp4_parser::Mp4Parser;
use uuid::Uuid;

pub struct MultimediaManagementService {
    blob: Box<dyn BlobStorageConnector>,
    video_repo: Box<dyn VideoTrackRepository>,
    audio_repo: Box<dyn AudioTrackRepository>,
    subtitle_repo: Box<dyn SubtitleTrackRepository>,
    container_repo: Box<dyn ContainerMetaRepository>,
}

impl MultimediaManagementService {
    pub fn new(
        blob: Box<dyn BlobStorageConnector>,
        video_repo: Box<dyn VideoTrackRepository>,
        audio_repo: Box<dyn AudioTrackRepository>,
        subtitle_repo: Box<dyn SubtitleTrackRepository>,
        container_repo: Box<dyn ContainerMetaRepository>,
    ) -> Self {
        MultimediaManagementService {
            blob,
            video_repo,
            audio_repo,
            subtitle_repo,
            container_repo,
        }
    }
}

#[async_trait]
impl MultimediaManagementServiceTrait for MultimediaManagementService {
    async fn upload_bytes(
        &self,
        blob_name: &str,
        data: &[u8],
        tags: Vec<Option<String>>,
    ) -> Result<ContainerMeta, Box<dyn std::error::Error + Send + Sync>> {
        let container_meta_id = Uuid::new_v4();
        let full_blob_name = format!("{}/{}", container_meta_id, blob_name);

        let tmp_path = format!("/tmp/{}", blob_name);
        tokio::fs::write(&tmp_path, data).await?;
        self.blob.upload_bytes(&full_blob_name, data).await?;

        let result = Mp4Parser::parse_from_file(&tmp_path)?;
        let _ = tokio::fs::remove_file(&tmp_path).await;
        let mut container_meta = result.container;

        if let Some(mut t) = result.video {
            t.id = Uuid::new_v4();
            t.container_meta_id = container_meta_id;
            self.video_repo.insert(&t).await?;
            container_meta.video_track_id = Some(t.id);
        }
        if let Some(mut t) = result.audio {
            t.id = Uuid::new_v4();
            t.container_meta_id = container_meta_id;
            self.audio_repo.insert(&t).await?;
            container_meta.audio_track_id = Some(t.id);
        }
        if let Some(mut t) = result.subtitle {
            t.id = Uuid::new_v4();
            t.container_meta_id = container_meta_id;
            self.subtitle_repo.insert(&t).await?;
            container_meta.subtitle_track_id = Some(t.id);
        }

        let now = Utc::now();
        container_meta.title = blob_name.to_string();
        container_meta.id = container_meta_id;
        container_meta.date_time_created = now;
        container_meta.date_time_updated = now;
        container_meta.tags = tags;

        self.container_repo.insert(&container_meta).await?;
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
        Ok(self.blob.download(&full_blob_name).await?)
    }

    async fn get_container_meta(
        &self,
        id: &Uuid,
    ) -> Result<ContainerMeta, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.container_repo.get_by_id(id).await?)
    }

    async fn get_video_track(
        &self,
        id: &Uuid,
    ) -> Result<VideoTrack, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.video_repo.get_by_id(id).await?)
    }

    async fn get_audio_track(
        &self,
        id: &Uuid,
    ) -> Result<AudioTrack, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.audio_repo.get_by_id(id).await?)
    }

    async fn get_subtitle_track(
        &self,
        id: &Uuid,
    ) -> Result<SubtitleTrack, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.subtitle_repo.get_by_id(id).await?)
    }

    async fn delete(
        &self,
        container_meta_id: &str,
        blob_name: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let full_blob_name = format!("{}/{}", container_meta_id, blob_name);
        self.blob.delete(&full_blob_name).await?;
        let id = Uuid::parse_str(container_meta_id)?;
        self.container_repo.delete(&id).await?;
        info!("Deleted blob and metadata for {}", id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use blob_storage_connector::aws_s3_connector::{AwsS3BucketConfig, AwsS3Connector};
    use persistence::psql_repository::{PsqlConfig, PsqlRepository};

    async fn setup_service() -> MultimediaManagementService {
        let _ = env_logger::try_init();

        let blob = AwsS3Connector::new(AwsS3BucketConfig {
            bucket_name: String::from("test-bucket"),
            endpoint_url: String::from("http://127.0.0.1:4566"),
            region: String::from("us-east-1"),
            access_key_id: String::from("test"),
            secret_access_key: String::from("test"),
        })
        .await;
        blob.create_bucket().await.ok();

        let repo = PsqlRepository::new(PsqlConfig {
            database_url: String::from("postgres://user:password@localhost:5432/diesel-demo"),
        })
        .await
        .expect("Failed to connect");

        MultimediaManagementService::new(
            Box::new(blob),
            Box::new(repo.clone()),
            Box::new(repo.clone()),
            Box::new(repo.clone()),
            Box::new(repo),
        )
    }

    #[tokio::test]
    async fn test_upload_bytes_download_delete() -> Result<(), Box<dyn std::error::Error>> {
        let service = setup_service().await;

        let blob_name = "nature.mp4";
        let data = std::fs::read("../../assets/nature.mp4")
            .map_err(|e| format!("failed to read asset: {}", e))?;
        let tags = vec![
            Some(String::from("nature")),
            Some(String::from("adventure")),
        ];

        let meta = service
            .upload_bytes(blob_name, &data, tags)
            .await
            .map_err(|e| format!("upload_bytes failed: {}", e))?;

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

        let bytes = service
            .download(&meta.id.to_string(), blob_name)
            .await
            .map_err(|e| format!("download failed: {}", e))?;
        assert!(!bytes.is_empty());

        service
            .delete(&meta.id.to_string(), blob_name)
            .await
            .map_err(|e| format!("delete failed: {}", e))?;

        Ok(())
    }
}
