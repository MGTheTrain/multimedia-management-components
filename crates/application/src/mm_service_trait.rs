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
use domain::{
    container_meta::ContainerMeta,
    tracks::{AudioTrack, SubtitleTrack, VideoTrack},
};
use uuid::Uuid;

#[async_trait]
pub trait MultimediaManagementServiceTrait {
    async fn upload_bytes(
        &self,
        blob_name: &str,
        data: &[u8],
        tags: Vec<Option<String>>,
    ) -> Result<ContainerMeta, Box<dyn std::error::Error + Send + Sync>>;
    async fn download(
        &self,
        container_meta_id: &str,
        blob_name: &str,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>>;
    async fn delete(
        &self,
        container_meta_id: &str,
        blob_name: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn get_container_meta(
        &self,
        id: &Uuid,
    ) -> Result<ContainerMeta, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_video_track(
        &self,
        id: &Uuid,
    ) -> Result<VideoTrack, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_audio_track(
        &self,
        id: &Uuid,
    ) -> Result<AudioTrack, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_subtitle_track(
        &self,
        id: &Uuid,
    ) -> Result<SubtitleTrack, Box<dyn std::error::Error + Send + Sync>>;
}
