FROM debian:stretch-slim as build

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH

RUN set -eux; \
    apt-get update; \
    apt-get install -y --no-install-recommends \
        ca-certificates \
        gcc \
        libc6-dev \
        wget \
        ; \
    \
    url="https://static.rust-lang.org/rustup/dist/x86_64-unknown-linux-gnu/rustup-init"; \
    wget "$url"; \
    chmod +x rustup-init; \
    ./rustup-init -y --no-modify-path --default-toolchain nightly-2019-06-01; \
    rm rustup-init; \
    chmod -R a+w $RUSTUP_HOME $CARGO_HOME; \
    rustup --version; \
    cargo --version; \
    rustc --version; \
    \
    apt-get remove -y --auto-remove \
        wget \
        ; \
    rm -rf /var/lib/apt/lists/*;

RUN USER=root cargo new blog

WORKDIR /blog

ADD Cargo.toml /blog/Cargo.toml
RUN cargo build --release
RUN rm src/*.rs

COPY src src
RUN touch src/*.rs
RUN cargo build --release

FROM ubuntu:rolling

WORKDIR /blog
COPY --from=build /blog/target/release/blog blog
COPY templates templates

WORKDIR /
EXPOSE 80 8000
CMD "/blog/blog"
