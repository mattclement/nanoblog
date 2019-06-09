FROM rustlang/rust:nightly as build
RUN USER=root cargo new blog

ENV blog_version 0.1.2
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
EXPOSE 80
CMD "/blog/blog"
