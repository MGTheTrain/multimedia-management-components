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

use crate::{
    container_meta::ContainerMeta,
    tracks::{AudioTrack, SubtitleTrack, VideoTrack},
};
use async_trait::async_trait;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("Not found")]
    NotFound,
    #[error("Database error: {0}")]
    Database(String),
    #[error("Validation error: {0}")]
    Validation(String),
}

#[async_trait]
pub trait VideoTrackRepository: Send + Sync {
    async fn insert(&self, track: &VideoTrack) -> Result<VideoTrack, RepositoryError>;
    async fn get_by_id(&self, id: &Uuid) -> Result<VideoTrack, RepositoryError>;
    async fn update(&self, id: &Uuid, track: &VideoTrack) -> Result<VideoTrack, RepositoryError>;
    async fn delete(&self, id: &Uuid) -> Result<(), RepositoryError>;
}

#[async_trait]
pub trait AudioTrackRepository: Send + Sync {
    async fn insert(&self, track: &AudioTrack) -> Result<AudioTrack, RepositoryError>;
    async fn get_by_id(&self, id: &Uuid) -> Result<AudioTrack, RepositoryError>;
    async fn update(&self, id: &Uuid, track: &AudioTrack) -> Result<AudioTrack, RepositoryError>;
    async fn delete(&self, id: &Uuid) -> Result<(), RepositoryError>;
}

#[async_trait]
pub trait SubtitleTrackRepository: Send + Sync {
    async fn insert(&self, track: &SubtitleTrack) -> Result<SubtitleTrack, RepositoryError>;
    async fn get_by_id(&self, id: &Uuid) -> Result<SubtitleTrack, RepositoryError>;
    async fn update(
        &self,
        id: &Uuid,
        track: &SubtitleTrack,
    ) -> Result<SubtitleTrack, RepositoryError>;
    async fn delete(&self, id: &Uuid) -> Result<(), RepositoryError>;
}

#[async_trait]
pub trait ContainerMetaRepository: Send + Sync {
    async fn insert(&self, meta: &ContainerMeta) -> Result<ContainerMeta, RepositoryError>;
    async fn get_by_id(&self, id: &Uuid) -> Result<ContainerMeta, RepositoryError>;
    async fn update(
        &self,
        id: &Uuid,
        meta: &ContainerMeta,
    ) -> Result<ContainerMeta, RepositoryError>;
    async fn delete(&self, id: &Uuid) -> Result<(), RepositoryError>;
}
