ARG RUSTC_VERSION=1.75.0

FROM rust:${RUSTC_VERSION}

WORKDIR /usr/lib/tempu-backend

COPY Cargo.toml Cargo.lock .env ./
COPY settings ./settings
COPY /src ./src 

RUN cargo install --path .
EXPOSE 8080
CMD ["backend"]