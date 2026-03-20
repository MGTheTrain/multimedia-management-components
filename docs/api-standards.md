# API Standards

## REST

The REST API follows [RFC 9110 (HTTP Semantics)](https://www.rfc-editor.org/rfc/rfc9110) and [RFC 9112 (HTTP/1.1)](https://www.rfc-editor.org/rfc/rfc9112):

- `POST /blobs` returns `201 Created` with a `Location` header pointing to the created resource per RFC 9110 §10.2.2
- `DELETE /blobs/{id}` returns `204 No Content` per RFC 9110 §15.3.5
- File upload uses `Content-Disposition` header per [RFC 6266](https://www.rfc-editor.org/rfc/rfc6266) instead of a query parameter
- Path parameters use short, resource-scoped names (`id` rather than `container_meta_id`) per REST resource naming conventions

## gRPC

The gRPC API follows [Google API Improvement Proposals (AIPs)](https://google.aip.dev/):

- Custom methods (`UploadBlob`, `DownloadBlob`) follow [AIP-136](https://google.aip.dev/136)
- Standard methods (`GetBlob`, `DeleteBlob`) follow [AIP-131](https://google.aip.dev/131) and [AIP-135](https://google.aip.dev/135)
- `UploadBlob` uses client-side streaming (`stream UploadBlobRequest`) with a `oneof` first message carrying metadata and subsequent messages carrying binary chunks. The production pattern for large file uploads
