FROM rust:1.94-slim AS builder

RUN apt-get update && apt-get install -y \
    libpq-dev \
    libssl-dev \
    pkg-config \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .
RUN cargo build --release --bin grpc-api

FROM rust:1.94-slim AS runtime

RUN apt-get update && apt-get install -y \
    libpq5 \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/grpc-api .

ENV RUST_LOG=info
EXPOSE 50051
CMD ["./grpc-api"]