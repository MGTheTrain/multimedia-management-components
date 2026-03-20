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
use std::sync::Arc;
use tonic::{Request, Response, Status, Streaming};
use uuid::Uuid;

pub mod multimedia {
    tonic::include_proto!("multimedia");
}

use multimedia::multimedia_service_server::MultimediaService;
use multimedia::*;

pub struct MultimediaGrpcService {
    pub service: Arc<dyn MultimediaManagementServiceTrait + Send + Sync>,
}

#[tonic::async_trait]
impl MultimediaService for MultimediaGrpcService {
    async fn upload_blob(
        &self,
        request: Request<Streaming<UploadBlobRequest>>,
    ) -> Result<Response<UploadBlobResponse>, Status> {
        let mut stream = request.into_inner();

        let mut blob_name = String::new();
        let mut tags: Vec<Option<String>> = vec![];
        let mut data: Vec<u8> = vec![];

        while let Some(msg) = stream.message().await? {
            match msg.data {
                Some(upload_blob_request::Data::Info(info)) => {
                    blob_name = info.blob_name;
                    tags = info.tags.into_iter().map(Some).collect();
                }
                Some(upload_blob_request::Data::Chunk(chunk)) => {
                    data.extend_from_slice(&chunk);
                }
                None => {}
            }
        }

        if blob_name.is_empty() {
            return Err(Status::invalid_argument("BlobInfo must be sent first"));
        }

        self.service
            .upload_bytes(&blob_name, &data, tags)
            .await
            .map(|meta| {
                Response::new(UploadBlobResponse {
                    id: meta.id.to_string(),
                })
            })
            .map_err(|e| Status::internal(e.to_string()))
    }

    async fn download_blob(
        &self,
        request: Request<DownloadBlobRequest>,
    ) -> Result<Response<DownloadBlobResponse>, Status> {
        let req = request.into_inner();
        let id = Uuid::parse_str(&req.id).map_err(|e| Status::invalid_argument(e.to_string()))?;
        let meta = self
            .service
            .get_container_meta(&id)
            .await
            .map_err(|e| Status::not_found(e.to_string()))?;
        self.service
            .download(&req.id, &meta.title)
            .await
            .map(|data| Response::new(DownloadBlobResponse { data }))
            .map_err(|e| Status::not_found(e.to_string()))
    }

    async fn get_blob(
        &self,
        request: Request<GetBlobRequest>,
    ) -> Result<Response<ContainerMetaResponse>, Status> {
        let id = Uuid::parse_str(&request.into_inner().id)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        self.service
            .get_container_meta(&id)
            .await
            .map(|meta| {
                Response::new(ContainerMetaResponse {
                    id: meta.id.to_string(),
                    title: meta.title,
                    duration: meta.duration,
                    file_size_in_kb: meta.file_size_in_kb,
                    tags: meta.tags.into_iter().flatten().collect(),
                })
            })
            .map_err(|e| Status::not_found(e.to_string()))
    }

    async fn delete_blob(
        &self,
        request: Request<DeleteBlobRequest>,
    ) -> Result<Response<DeleteBlobResponse>, Status> {
        let req = request.into_inner();
        let id = Uuid::parse_str(&req.id).map_err(|e| Status::invalid_argument(e.to_string()))?;
        let meta = self
            .service
            .get_container_meta(&id)
            .await
            .map_err(|e| Status::not_found(e.to_string()))?;
        self.service
            .delete(&req.id, &meta.title)
            .await
            .map(|_| Response::new(DeleteBlobResponse {}))
            .map_err(|e| Status::internal(e.to_string()))
    }
}
