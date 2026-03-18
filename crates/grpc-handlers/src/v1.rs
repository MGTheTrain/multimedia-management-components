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
use tonic::{Request, Response, Status};
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
        request: Request<UploadBlobRequest>,
    ) -> Result<Response<UploadBlobResponse>, Status> {
        let req = request.into_inner();
        let tags = req.tags.into_iter().map(Some).collect();
        self.service
            .upload_bytes(&req.blob_name, &req.data, tags)
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
        let id = Uuid::parse_str(&req.container_meta_id)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        let meta = self
            .service
            .get_container_meta(&id)
            .await
            .map_err(|e| Status::not_found(e.to_string()))?;
        self.service
            .download(&req.container_meta_id, &meta.title)
            .await
            .map(|data| Response::new(DownloadBlobResponse { data }))
            .map_err(|e| Status::not_found(e.to_string()))
    }

    async fn delete_blob(
        &self,
        request: Request<DeleteBlobRequest>,
    ) -> Result<Response<DeleteBlobResponse>, Status> {
        let req = request.into_inner();
        let id = Uuid::parse_str(&req.container_meta_id)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        let meta = self
            .service
            .get_container_meta(&id)
            .await
            .map_err(|e| Status::not_found(e.to_string()))?;
        self.service
            .delete(&req.container_meta_id, &meta.title)
            .await
            .map(|_| Response::new(DeleteBlobResponse {}))
            .map_err(|e| Status::internal(e.to_string()))
    }

    async fn get_container_meta(
        &self,
        request: Request<GetContainerMetaRequest>,
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
}
