#!/bin/bash
set -e

GRPC_URL="localhost:50051"
PROTO="api/multimedia.proto"
ASSET="assets/nature.mp4"
BLOB_NAME="nature.mp4"
TMP_PAYLOAD="/tmp/grpc_upload_payload.json"

echo "=== Upload ==="
# macOS uses -i for input file, Linux uses -w 0 to disable line wrapping
if [[ "$OSTYPE" == "darwin"* ]]; then
  BASE64_DATA=$(base64 -i "$ASSET")
else
  BASE64_DATA=$(base64 -w 0 "$ASSET")
fi
echo "{\"blob_name\": \"$BLOB_NAME\", \"data\": \"$BASE64_DATA\", \"tags\": [\"nature\"]}" >"$TMP_PAYLOAD"
UPLOAD_RESPONSE=$(grpcurl -plaintext -proto "$PROTO" \
  -d @ "$GRPC_URL" multimedia.MultimediaService/UploadBlob <"$TMP_PAYLOAD")
echo "Response: $UPLOAD_RESPONSE"

CONTAINER_META_ID=$(echo "$UPLOAD_RESPONSE" | grep -o '"id": "[^"]*"' | cut -d'"' -f4)
echo "Container meta ID: $CONTAINER_META_ID"
if [ -z "$CONTAINER_META_ID" ]; then
  echo "Upload failed!" && exit 1
fi

echo ""
echo "=== Get Container Meta ==="
META_RESPONSE=$(grpcurl -plaintext -proto "$PROTO" \
  -d "{\"id\": \"$CONTAINER_META_ID\"}" \
  "$GRPC_URL" multimedia.MultimediaService/GetContainerMeta)
echo "Response: $META_RESPONSE"

echo ""
echo "=== Download ==="
DOWNLOAD_RESPONSE=$(grpcurl -plaintext -proto "$PROTO" \
  -max-msg-sz 104857600 \
  -d "{\"container_meta_id\": \"$CONTAINER_META_ID\"}" \
  "$GRPC_URL" multimedia.MultimediaService/DownloadBlob)

echo "$DOWNLOAD_RESPONSE" | grep -o '"data": "[^"]*"' | cut -d'"' -f4 | base64 -d >/tmp/downloaded_nature.mp4
echo "Downloaded to /tmp/downloaded_nature.mp4 ($(wc -c </tmp/downloaded_nature.mp4) bytes)"

ORIGINAL_SIZE=$(wc -c <"$ASSET")
DOWNLOADED_SIZE=$(wc -c </tmp/downloaded_nature.mp4)
echo "Original size:   $ORIGINAL_SIZE bytes"
echo "Downloaded size: $DOWNLOADED_SIZE bytes"
if [ "$ORIGINAL_SIZE" != "$DOWNLOADED_SIZE" ]; then
  echo "Size mismatch — download corrupted!" && exit 1
fi
echo "Size check passed"

echo ""
echo "=== Delete ==="
grpcurl -plaintext -proto "$PROTO" \
  -d "{\"container_meta_id\": \"$CONTAINER_META_ID\"}" \
  "$GRPC_URL" multimedia.MultimediaService/DeleteBlob
echo "Deleted successfully"

rm -f "$TMP_PAYLOAD"
echo ""
echo "All gRPC endpoints passed."
