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
    async fn upload_blob(
        &self,
        _method: &Method,
        _host: &Host,
        _cookies: &CookieJar,
        query_params: &openapi::models::UploadBlobQueryParams,
        body: &Bytes,
    ) -> Result<openapi::apis::default::UploadBlobResponse, ()> {
        let tags = vec![Some(String::from("uploaded"))];
        match self
            .service
            .upload_bytes(&query_params.filename, body, tags)
            .await
        {
            Ok(meta) => Ok(
                openapi::apis::default::UploadBlobResponse::Status201_Created(
                    openapi::models::UploadResponse { id: meta.id },
                ),
            ),
            Err(_) => Ok(openapi::apis::default::UploadBlobResponse::Status400_BadRequest),
        }
    }

    async fn download_blob(
        &self,
        _method: &Method,
        _host: &Host,
        _cookies: &CookieJar,
        path_params: &openapi::models::DownloadBlobPathParams,
    ) -> Result<openapi::apis::default::DownloadBlobResponse, ()> {
        let id = match uuid::Uuid::parse_str(&path_params.container_meta_id) {
            Ok(id) => id,
            Err(_) => return Ok(openapi::apis::default::DownloadBlobResponse::Status404_NotFound),
        };
        let meta = match self.service.get_container_meta(&id).await {
            Ok(m) => m,
            Err(_) => return Ok(openapi::apis::default::DownloadBlobResponse::Status404_NotFound),
        };
        match self
            .service
            .download(&path_params.container_meta_id, &meta.title)
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
        let id = match uuid::Uuid::parse_str(&path_params.container_meta_id) {
            Ok(id) => id,
            Err(_) => return Ok(openapi::apis::default::DeleteBlobResponse::Status404_NotFound),
        };
        let meta = match self.service.get_container_meta(&id).await {
            Ok(m) => m,
            Err(_) => return Ok(openapi::apis::default::DeleteBlobResponse::Status404_NotFound),
        };
        match self
            .service
            .delete(&path_params.container_meta_id, &meta.title)
            .await
        {
            Ok(_) => Ok(openapi::apis::default::DeleteBlobResponse::Status204_NoContent),
            Err(_) => Ok(openapi::apis::default::DeleteBlobResponse::Status404_NotFound),
        }
    }
}

// TODO: setup e2e-test with axum-test
