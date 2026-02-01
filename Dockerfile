FROM rust:slim-bookworm AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    g++ \
    cmake \
    make \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy manifests first for better caching
COPY Cargo.toml Cargo.lock ./

# Create dummy src to build dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    echo "pub fn dummy() {}" > src/lib.rs

RUN cargo build --release && \
    rm -rf src

# Copy actual source and build
COPY src ./src

RUN touch src/main.rs src/lib.rs && \
    cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/shizu /usr/local/bin/shizu

ENV HOST=0.0.0.0
ENV PORT=8080

EXPOSE 8080

CMD ["shizu"]
