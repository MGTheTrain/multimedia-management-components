use async_trait::async_trait;
use axum::extract::*;
use axum_extra::extract::CookieJar;
use bytes::Bytes;
use headers::Host;
use http::Method;
use serde::{Deserialize, Serialize};

use crate::{models, types::*};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum DeleteBlobResponse {
    /// No Content
    Status204_NoContent,
    /// Not Found
    Status404_NotFound,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum DownloadBlobResponse {
    /// OK
    Status200_OK(ByteArray),
    /// Not Found
    Status404_NotFound,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum UploadBlobResponse {
    /// Created
    Status201_Created(models::UploadResponse),
    /// Bad Request
    Status400_BadRequest,
}

/// Default
#[async_trait]
#[allow(clippy::ptr_arg)]
pub trait Default<E: std::fmt::Debug + Send + Sync + 'static = ()>: super::ErrorHandler<E> {
    /// Delete a blob.
    ///
    /// DeleteBlob - DELETE /blobs/{container_meta_id}
    async fn delete_blob(
        &self,

        method: &Method,
        host: &Host,
        cookies: &CookieJar,
        path_params: &models::DeleteBlobPathParams,
    ) -> Result<DeleteBlobResponse, E>;

    /// Download a blob.
    ///
    /// DownloadBlob - GET /blobs/{container_meta_id}
    async fn download_blob(
        &self,

        method: &Method,
        host: &Host,
        cookies: &CookieJar,
        path_params: &models::DownloadBlobPathParams,
    ) -> Result<DownloadBlobResponse, E>;

    /// Upload a blob.
    ///
    /// UploadBlob - POST /blobs
    async fn upload_blob(
        &self,

        method: &Method,
        host: &Host,
        cookies: &CookieJar,
        query_params: &models::UploadBlobQueryParams,
        body: &Bytes,
    ) -> Result<UploadBlobResponse, E>;
}
