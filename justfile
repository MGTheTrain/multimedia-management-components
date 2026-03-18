# Run tests for a given module, e.g. `just test domain`
test module:
    cargo test --manifest-path crates/{{module}}/Cargo.toml

# Run coverage for a given module, e.g. `just coverage infrastructure/persistence`
coverage module:
    cargo tarpaulin --manifest-path crates/{{module}}/Cargo.toml --out Html

# Update dependencies for a given module, e.g. `just upgrade infrastructure/persistence`
upgrade module:
    cargo upgrade --manifest-path crates/{{module}}/Cargo.toml --incompatible

# Format Rust files
format:
    cargo fmt

# Check Rust formatting without modifying files
format-check:
    cargo fmt --check

# Lint Rust files (warnings treated as errors)
lint:
    cargo clippy -- -D warnings

# Start infrastructure containers (postgres, localstack, azurite)
compose-start:
    docker compose -f infra/compose/docker-compose.yml up -d

# Start all services including REST API
compose-start-api:
    docker compose -f infra/compose/docker-compose.yml --profile api up -d --build

# Stop and remove docker containers
compose-stop:
    docker compose -f infra/compose/docker-compose.yml --profile api down -v

# Start REST API server locally
run-rest-api:
    cargo run --release --bin rest-api

# Run S3 CLI, e.g. `just s3-cli create-bucket`
s3-cli *args:
    cargo run --release --bin s3-cli -- {{args}}

# Run Azure CLI, e.g. `just azure-cli create-container`
azure-cli *args:
    cargo run --release --bin azure-cli -- {{args}}

# Test REST API endpoints
test-rest-api:
    bash scripts/test_rest_api.sh

# Generate gRPC and REST server code from api/ specs
generate-server-stubs:
    bash scripts/generate.sh

# Start gRPC API server locally
run-grpc-api:
    cargo run --release --bin grpc-api

# Test gRPC endpoints
test-grpc-api:
    bash scripts/test_grpc_api.sh

# Remove all Rust target build directories
clean:
    find . -type d -name target -exec rm -rf {} +