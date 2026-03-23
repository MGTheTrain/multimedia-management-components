# Run tests for a given module, e.g. `just test domain`
test module:
    cargo test --manifest-path crates/{{module}}/Cargo.toml

# Update dependencies for a given module, e.g. `just upgrade infrastructure/persistence`
upgrade module:
    cargo upgrade --manifest-path crates/{{module}}/Cargo.toml --incompatible

# Generate and open documentation for a given module
doc module:
    cargo doc --manifest-path crates/{{module}}/Cargo.toml --no-deps

# Package release binaries into a distributable archive
package revision="":
    #!/usr/bin/env bash
    set -euo pipefail
    VERSION=$(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)
    PKG_NAME="rust-app-${VERSION}${revision:+-${revision}}"
    mkdir -p packages
    zip -j "packages/${PKG_NAME}-linux-x64.zip" \
      target/release/rest-api \
      target/release/grpc-api
    echo "Packaged: packages/${PKG_NAME}-linux-x64.zip"

# Run tests for all modules
test-all:
    just test domain
    just test parsers
    just test infrastructure/blob-storage-connector
    just test infrastructure/persistence
    just test application

# Run coverage for all modules (HTML, Cobertura XML and LCOV)
coverage:
    cargo tarpaulin --manifest-path crates/domain/Cargo.toml --include-files "crates/domain/*" --out Xml Html Lcov --output-dir coverage/domain
    cargo tarpaulin --manifest-path crates/parsers/Cargo.toml --include-files "crates/parsers/*" --out Xml Html Lcov --output-dir coverage/parsers
    cargo tarpaulin --manifest-path crates/infrastructure/blob-storage-connector/Cargo.toml --include-files "crates/infrastructure/blob-storage-connector/*" --out Xml Html Lcov --output-dir coverage/blob-storage-connector
    cargo tarpaulin --manifest-path crates/infrastructure/persistence/Cargo.toml --include-files "crates/infrastructure/persistence/*" --out Xml Html Lcov --output-dir coverage/persistence
    cargo tarpaulin --manifest-path crates/application/Cargo.toml --include-files "crates/application/*" --out Xml Html Lcov --output-dir coverage/application

# Run SonarCloud analysis and check quality thresholds
sonar:
    bash scripts/sonar.sh

# Format Rust files
format:
    cargo fmt

# Run pre-commit hooks on all files (includes fmt, clippy, shfmt, codespell)
pre-commit:
    pre-commit run --all-files

# Install pre-commit hooks
pre-commit-install:
    pre-commit install

# Start infrastructure containers (postgres, localstack, azurite)
compose-start:
    docker compose -f infra/compose/docker-compose.yml up -d

# Start all services including REST and gRPC APIs
compose-start-api:
    docker compose -f infra/compose/docker-compose.yml --profile api up -d --build

# Stop and remove docker containers
compose-stop:
    docker compose -f infra/compose/docker-compose.yml --profile api down -v

# Start REST API server locally
run-rest-api:
    cargo run --release --bin rest-api

# Start gRPC API server locally
run-grpc-api:
    cargo run --release --bin grpc-api

# Run S3 CLI, e.g. `just s3-cli create-bucket`
s3-cli *args:
    cargo run --release --bin s3-cli -- {{args}}

# Run Azure CLI, e.g. `just azure-cli create-container`
azure-cli *args:
    cargo run --release --bin azure-cli -- {{args}}

# Test REST API endpoints
test-rest-api:
    bash scripts/test_rest_api.sh

# Test gRPC endpoints
test-grpc-api:
    bash scripts/test_grpc_api.sh

# Generate gRPC and REST server code from api/ specs
generate-server-stubs:
    bash scripts/generate.sh

# Remove all Rust target build directories and packages
clean:
    find . -type d -name target -exec rm -rf {} +
    rm -rf packages/ .scannerwork/
