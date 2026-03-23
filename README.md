# Multimedia Management Components

### Summary

Experimental Rust-based multimedia management components providing a REST and gRPC API for uploading, storing and retrieving multimedia files with parsed track metadata, alongside CLI tools for direct blob storage operations on AWS S3 and Azure Blob Storage.

**Binaries:**
- `rest-api` — HTTP REST API backed by PostgreSQL and S3-compatible storage
- `grpc-api` — gRPC API backed by PostgreSQL and S3-compatible storage
- `s3-cli` — CLI for AWS S3 blob and bucket operations
- `azure-cli` — CLI for Azure Blob Storage container and blob operations

### Architecture

This project follows Clean Architecture with strict dependency inversion:

```
domain          — entities, repository traits, blob storage trait
application     — service trait and implementation, orchestrates domain
infrastructure  — persistence (Diesel/PostgreSQL), blob storage (S3/Azure)
rest-handlers   — implements generated OpenAPI server trait (rust-axum)
grpc-handlers   — implements generated gRPC server trait (tonic)
rest-api        — binary, wires REST dependencies
grpc-api        — binary, wires gRPC dependencies
```

### API First

REST and gRPC interfaces are defined in `api/` and server code is generated from them:

```
api/openapi.json       → openapi-generator (rust-axum) → crates/rest-handlers/generated/
api/multimedia.proto   → tonic_prost_build (build.rs)  → generated at compile time
```

To regenerate:
```sh
just generate-server-stubs
```

#### Ports
| Service  | Port  |
|----------|-------|
| REST API | 8080  |
| gRPC API | 50051 |
| Postgres | 5432  |
| LocalStack (S3) | 4566 |
| Azurite (Azure) | 10000 |


### Code Quality

pre-commit (formatting and linting — no build required):
```sh
pip install pre-commit
just pre-commit
```

SonarCloud (coverage and static analysis — requires a SonarCloud account):
```sh
export SONAR_TOKEN=<your-token>
just sonar
```

Results are published to [sonarcloud.io](https://sonarcloud.io/project/overview?id=MGTheTrain_multimedia-management-components). Both checks run automatically in CI on every pull request and push to main.

### Quick Start

#### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [just](https://github.com/casey/just)
- [Docker](https://docs.docker.com/get-docker/)

#### Just recipes
```sh
just --list
Available recipes:
    azure-cli *args       # Run Azure CLI, e.g. `just azure-cli create-container`
    clean                 # Remove all Rust target build directories and packages
    compose-start         # Start infrastructure containers (postgres, localstack, azurite)
    compose-start-api     # Start all services including REST and gRPC APIs
    compose-stop          # Stop and remove docker containers
    coverage              # Run coverage for all modules (HTML, Cobertura XML and LCOV)
    doc module            # Generate and open documentation for a given module
    format                # Format Rust files
    generate-server-stubs # Generate gRPC and REST server code from api/ specs
    package revision=""   # Package release binaries into a distributable archive
    pre-commit            # Run pre-commit hooks on all files (includes fmt, clippy, shfmt, codespell)
    pre-commit-install    # Install pre-commit hooks
    run-grpc-api          # Start gRPC API server locally
    run-rest-api          # Start REST API server locally
    s3-cli *args          # Run S3 CLI, e.g. `just s3-cli create-bucket`
    sonar                 # Run SonarCloud analysis and check quality thresholds
    test module           # Run tests for a given module, e.g. `just test domain`
    test-all              # Run tests for all modules
    test-grpc-api         # Test gRPC endpoints
    test-rest-api         # Test REST API endpoints
    upgrade module        # Update dependencies for a given module, e.g. `just upgrade infrastructure/persistence`
```

#### Running REST API locally
```sh
just compose-start

export AWS_BUCKET_NAME="test-bucket"
export AWS_ENDPOINT_URL="http://127.0.0.1:4566"
export AWS_DEFAULT_REGION="us-east-1"
export AWS_ACCESS_KEY_ID="test"
export AWS_SECRET_ACCESS_KEY="test"
export DATABASE_URL="postgres://user:password@localhost:5432/diesel-demo"

RUST_LOG=info just run-rest-api
just test-rest-api # Test REST API endpoints
```

#### Running gRPC API locally
```sh
just compose-start

export AWS_BUCKET_NAME="test-bucket"
export AWS_ENDPOINT_URL="http://127.0.0.1:4566"
export AWS_DEFAULT_REGION="us-east-1"
export AWS_ACCESS_KEY_ID="test"
export AWS_SECRET_ACCESS_KEY="test"
export DATABASE_URL="postgres://user:password@localhost:5432/diesel-demo"

RUST_LOG=info just run-grpc-api
just test-grpc-api # Test gRPC API endpoints
```

#### Running REST and gRPC APIs via Docker
```sh
just compose-start-api
just test-rest-api # Test REST API endpoints
just test-grpc-api # Test REST gRPC endpoints
```

#### Invoking CLI commands
```sh
just compose-start

# AWS S3 (LocalStack)
export AWS_BUCKET_NAME="test-bucket"
export AWS_ENDPOINT_URL="http://127.0.0.1:4566"
export AWS_DEFAULT_REGION="us-east-1"
export AWS_ACCESS_KEY_ID="test"
export AWS_SECRET_ACCESS_KEY="test"

just s3-cli create-bucket
just s3-cli upload-blob --blob-name nature.mp4 --file-path assets/nature.mp4
just s3-cli download-blob --blob-name nature.mp4 --output-path /tmp/nature-s3.mp4
just s3-cli delete-blob --blob-name nature.mp4

# Azure Blob Storage (Azurite)
export AZURE_STORAGE_ACCOUNT="devstoreaccount1"
export AZURE_STORAGE_KEY="Eby8vdM02xNOcqFlqUwJPLlmEtlCDXJ1OUzFT50uSRZ6IFsuFq2UVErCz4I6tq/K1SZFPTOtr/KBHBeksoGMGw=="
export AZURE_CONTAINER_NAME="test-container"
export AZURE_ENDPOINT_URL="http://127.0.0.1:10000"

just azure-cli create-container
just azure-cli upload-blob --blob-name nature.mp4 --file-path assets/nature.mp4
just azure-cli download-blob --blob-name nature.mp4 --output-path /tmp/nature-azure.mp4
just azure-cli delete-blob --blob-name nature.mp4
```

### NOTES

#### Binary size

Rust's standard library and pure-Rust dependencies are statically linked by default. However, this project dynamically links to native C libraries, `libpq` (PostgreSQL client) and `libssl` (OpenSSL), via FFI. Those libraries must be present on the host at runtime (see [Docker image](./infra/docker/rest-api/Dockerfile) installing `libpq5` and `libssl3`).

```sh
cargo build --release --bin rest-api
ldd target/release/rest-api

# Output
# linux-vdso.so.1 (0x0000ffffa595c000)
# libpq.so.5 => /lib/aarch64-linux-gnu/libpq.so.5 (0x0000ffffa5040000)
# libgcc_s.so.1 => /lib/aarch64-linux-gnu/libgcc_s.so.1 (0x0000ffffa5000000)
# libm.so.6 => /lib/aarch64-linux-gnu/libm.so.6 (0x0000ffffa4f50000)
# libc.so.6 => /lib/aarch64-linux-gnu/libc.so.6 (0x0000ffffa4da0000)
# libssl.so.3 => /lib/aarch64-linux-gnu/libssl.so.3 (0x0000ffffa4ce0000)
# libcrypto.so.3 => /lib/aarch64-linux-gnu/libcrypto.so.3 (0x0000ffffa4880000)
# libgssapi_krb5.so.2 => /lib/aarch64-linux-gnu/libgssapi_krb5.so.2 (0x0000ffffa4810000)
# libldap-2.5.so.0 => /lib/aarch64-linux-gnu/libldap-2.5.so.0 (0x0000ffffa4790000)
# /lib/ld-linux-aarch64.so.1 (0x0000ffffa5920000)
# libkrb5.so.3 => /lib/aarch64-linux-gnu/libkrb5.so.3 (0x0000ffffa46a0000)
# libk5crypto.so.3 => /lib/aarch64-linux-gnu/libk5crypto.so.3 (0x0000ffffa4650000)
# libcom_err.so.2 => /lib/aarch64-linux-gnu/libcom_err.so.2 (0x0000ffffa4620000)
# libkrb5support.so.0 => /lib/aarch64-linux-gnu/libkrb5support.so.0 (0x0000ffffa45f0000)
# liblber-2.5.so.0 => /lib/aarch64-linux-gnu/liblber-2.5.so.0 (0x0000ffffa45c0000)
# libsasl2.so.2 => /lib/aarch64-linux-gnu/libsasl2.so.2 (0x0000ffffa4580000)
# libgnutls.so.30 => /lib/aarch64-linux-gnu/libgnutls.so.30 (0x0000ffffa4330000)
# libkeyutils.so.1 => /lib/aarch64-linux-gnu/libkeyutils.so.1 (0x0000ffffa4300000)
# libresolv.so.2 => /lib/aarch64-linux-gnu/libresolv.so.2 (0x0000ffffa42d0000)
# libp11-kit.so.0 => /lib/aarch64-linux-gnu/libp11-kit.so.0 (0x0000ffffa4180000)
# libidn2.so.0 => /lib/aarch64-linux-gnu/libidn2.so.0 (0x0000ffffa4130000)
# libunistring.so.2 => /lib/aarch64-linux-gnu/libunistring.so.2 (0x0000ffffa3f60000)
# libtasn1.so.6 => /lib/aarch64-linux-gnu/libtasn1.so.6 (0x0000ffffa3f20000)
# libnettle.so.8 => /lib/aarch64-linux-gnu/libnettle.so.8 (0x0000ffffa3eb0000)
# libhogweed.so.6 => /lib/aarch64-linux-gnu/libhogweed.so.6 (0x0000ffffa3e40000)
# libgmp.so.10 => /lib/aarch64-linux-gnu/libgmp.so.10 (0x0000ffffa3da0000)
# libffi.so.8 => /lib/aarch64-linux-gnu/libffi.so.8 (0x0000ffffa3d70000)
```

If `[profile.release]` is set to `debug = 2` to retain full debug symbols, this significantly inflates the binary. To produce a lean release binary, refer to following [Github repository on Minimizing Rust Binary Size](https://github.com/johnthagen/min-sized-rust) and set within the [Cargo.toml](./Cargo.toml) for `release` builds:

```toml
[profile.release]
panic = "abort"
strip = true
lto = "thin"
codegen-units = 1
opt-level = 3
```

For reference: [Why do Rust binaries are so huge?](https://users.rust-lang.org/t/why-do-rust-binaries-are-so-huge/124450/5)

### Documentation

Navigate to the [docs folder](./docs/).
