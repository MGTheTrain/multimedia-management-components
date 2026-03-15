#!/bin/bash
set -e

BASE_URL="http://localhost:8080"
ASSET="assets/nature.mp4"
BLOB_NAME="nature.mp4"

echo "=== Upload ==="
RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL/blobs?filename=$BLOB_NAME" \
  --data-binary "@$ASSET" \
  -H "Content-Type: video/mp4")
HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
BODY=$(echo "$RESPONSE" | head -n-1)
echo "Status: $HTTP_CODE"
echo "Body: $BODY"

if [ "$HTTP_CODE" != "201" ]; then
  echo "Upload failed!" && exit 1
fi

CONTAINER_META_ID=$(echo "$BODY" | grep -o '"id":"[^"]*"' | cut -d'"' -f4)
echo "Container meta ID: $CONTAINER_META_ID"

echo ""
echo "=== Download ==="
HTTP_CODE=$(curl -s -o /tmp/downloaded_nature.mp4 -w "%{http_code}" \
  "$BASE_URL/blobs/$CONTAINER_META_ID/$BLOB_NAME")
echo "Status: $HTTP_CODE"
if [ "$HTTP_CODE" != "200" ]; then
  echo "Download failed!" && exit 1
fi
echo "Downloaded to /tmp/downloaded_nature.mp4 ($(wc -c < /tmp/downloaded_nature.mp4) bytes)"

echo ""
echo "=== Delete ==="
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" -X DELETE \
  "$BASE_URL/blobs/$CONTAINER_META_ID/$BLOB_NAME")
echo "Status: $HTTP_CODE"
if [ "$HTTP_CODE" != "204" ]; then
  echo "Delete failed!" && exit 1
fi

echo ""
echo "All endpoints passed."