# syntax=docker/dockerfile:1.7-labs

ARG RUSTC_VERSION=1.75.0

################## Stage 1 ##################
FROM rust:${RUSTC_VERSION}-slim-buster as builder

RUN apt-get update && apt-get install -y openssl libssl-dev pkg-config

WORKDIR /usr/lib/tempusalert-be

RUN mkdir src && echo 'fn main() {}' > src/main.rs
RUN cargo fetch
RUN cargo build --release
RUN rm -rf src

COPY Cargo.toml Cargo.lock .env ./
COPY settings ./settings

COPY ./src ./src 

################## Stage 2 ##################
FROM rust:${RUSTC_VERSION}-slim-buster
COPY --from=builder /usr/lib/tempusalert-be/target/release/tempusalert-be .
CMD ["./tempusalert-be"]