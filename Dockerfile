FROM rustlang/rust:nightly
RUN USER=root cargo new blog

WORKDIR /blog
ADD Cargo.toml /blog/Cargo.toml
RUN cargo build --release
RUN rm src/*.rs

COPY src src
RUN touch src/*.rs
RUN cargo build --release && mv target/release/blog /bin && rm -rf /blog

WORKDIR /
EXPOSE 8000
CMD "/bin/blog"
