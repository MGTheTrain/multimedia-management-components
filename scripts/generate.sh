#!/bin/bash
set -e

echo "=== Generating gRPC Rust code from proto ==="
# tonic_build handles this via build.rs. Nothing to do here manually

echo "=== Generating REST server from OpenAPI spec ==="
openapi-generator-cli generate \
  -i api/openapi.json \
  -g rust-axum \
  -o crates/rest-handlers/generated/rest-server

echo "Done."
