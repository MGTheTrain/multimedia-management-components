FROM rust:1.94-slim AS builder

RUN apt-get update && apt-get install -y \
    libpq-dev \
    libssl-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .
RUN cargo build --release --bin rest-api

FROM rust:1.94-slim AS runtime

RUN apt-get update && apt-get install -y \
    libpq5 \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/rest-api .

ENV RUST_LOG=info
EXPOSE 8080
CMD ["./rest-api"]
