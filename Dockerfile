ARG RUSTC_VERSION=1.75.0

################## Stage 1 ##################
FROM rust:${RUSTC_VERSION}-slim-buster as builder

RUN apt-get update && apt-get install -y openssl libssl-dev pkg-config

WORKDIR /tempusalert-be

COPY Cargo.toml Cargo.lock ./

RUN mkdir -p src/bin/backend && echo 'fn main() {}' > src/bin/backend/main.rs
RUN cargo build --bin backend --release
RUN rm -rf src

COPY ./settings ./settings
COPY ./src ./src 
RUN cargo build --bin backend --release

################## Stage 2 ##################
FROM rust:${RUSTC_VERSION}-slim-buster
WORKDIR /tempusalert-be

COPY --from=builder /tempusalert-be/settings settings
COPY --from=builder /tempusalert-be/target/release/backend .

EXPOSE 8081
CMD ["./backend"]
