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

use application::mm_service_trait::MultimediaManagementServiceTrait;
use axum_extra::extract::CookieJar;
use bytes::Bytes;
use headers::Host;
use http::Method;
use std::sync::Arc;

pub type AppState = Arc<dyn MultimediaManagementServiceTrait + Send + Sync>;

pub struct MultimediaApi {
    pub service: AppState,
}

impl AsRef<MultimediaApi> for MultimediaApi {
    fn as_ref(&self) -> &MultimediaApi {
        self
    }
}

// ErrorHandler in openapi::apis
impl openapi::apis::ErrorHandler<()> for MultimediaApi {}

#[async_trait::async_trait]
impl openapi::apis::default::Default<()> for MultimediaApi {
    async fn get_blob_metadata(
        &self,
        _method: &Method,
        _host: &Host,
        _cookies: &CookieJar,
        path_params: &openapi::models::GetBlobMetadataPathParams,
    ) -> Result<openapi::apis::default::GetBlobMetadataResponse, ()> {
        // path_params.id is already uuid::Uuid — no parse needed
        match self.service.get_container_meta(&path_params.id).await {
            Ok(meta) => Ok(
                openapi::apis::default::GetBlobMetadataResponse::Status200_OK(
                    openapi::models::ContainerMetaResponse {
                        id: meta.id,
                        title: meta.title,
                        description: Some(meta.description),
                        tags: Some(meta.tags.into_iter().flatten().collect()),
                        file_size_in_kb: meta.file_size_in_kb,
                        duration: meta.duration,
                        date_time_created: Some(meta.date_time_created),
                        date_time_updated: Some(meta.date_time_updated),
                    },
                ),
            ),
            Err(_) => Ok(openapi::apis::default::GetBlobMetadataResponse::Status404_NotFound),
        }
    }

    async fn download_blob(
        &self,
        _method: &Method,
        _host: &Host,
        _cookies: &CookieJar,
        path_params: &openapi::models::DownloadBlobPathParams,
    ) -> Result<openapi::apis::default::DownloadBlobResponse, ()> {
        // path_params.id is uuid::Uuid directly
        let meta = match self.service.get_container_meta(&path_params.id).await {
            Ok(m) => m,
            Err(_) => return Ok(openapi::apis::default::DownloadBlobResponse::Status404_NotFound),
        };
        match self
            .service
            .download(&path_params.id.to_string(), &meta.title)
            .await
        {
            Ok(bytes) => Ok(openapi::apis::default::DownloadBlobResponse::Status200_OK(
                openapi::types::ByteArray(bytes),
            )),
            Err(_) => Ok(openapi::apis::default::DownloadBlobResponse::Status404_NotFound),
        }
    }

    async fn delete_blob(
        &self,
        _method: &Method,
        _host: &Host,
        _cookies: &CookieJar,
        path_params: &openapi::models::DeleteBlobPathParams,
    ) -> Result<openapi::apis::default::DeleteBlobResponse, ()> {
        let meta = match self.service.get_container_meta(&path_params.id).await {
            Ok(m) => m,
            Err(_) => return Ok(openapi::apis::default::DeleteBlobResponse::Status404_NotFound),
        };
        match self
            .service
            .delete(&path_params.id.to_string(), &meta.title)
            .await
        {
            Ok(_) => Ok(openapi::apis::default::DeleteBlobResponse::Status204_NoContent),
            Err(_) => Ok(openapi::apis::default::DeleteBlobResponse::Status404_NotFound),
        }
    }

    async fn upload_blob(
        &self,
        _method: &Method,
        _host: &Host,
        _cookies: &CookieJar,
        header_params: &openapi::models::UploadBlobHeaderParams,
        body: &Bytes,
    ) -> Result<openapi::apis::default::UploadBlobResponse, ()> {
        // content_disposition is String (required) — no Option unwrap needed
        let file_name = header_params
            .content_disposition
            .split(';')
            .find_map(|p| p.trim().strip_prefix("filename="))
            .map(|f| f.trim_matches('"').to_string())
            .unwrap_or_else(|| "upload".to_string());

        let tags = vec![Some(String::from("uploaded"))];
        match self.service.upload_bytes(&file_name, body, tags).await {
            Ok(meta) => Ok(
                openapi::apis::default::UploadBlobResponse::Status201_Created {
                    body: openapi::models::UploadResponse { id: meta.id },
                    location: Some(format!("/blobs/{}", meta.id)),
                },
            ),
            Err(_) => Ok(openapi::apis::default::UploadBlobResponse::Status400_BadRequest),
        }
    }
}

// TODO: setup e2e-test with axum-test
