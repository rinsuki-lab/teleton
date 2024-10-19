FROM rust:1.82.0-bookworm as builder

RUN apt-get update && apt-get install -y protobuf-compiler && apt-get clean && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY download_and_patch.py ./
RUN python3 download_and_patch.py

COPY Cargo.toml Cargo.lock ./

ENV RUSTFLAGS "--cfg aes_armv8"
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -r src target/release/deps/teleton-*

COPY build.rs ./
COPY proto/ ./proto/
COPY src/ ./src/
RUN cargo build --release

FROM debian:bookworm-slim

WORKDIR /app
COPY --from=builder /app/target/release/teleton .

CMD ["./teleton"]