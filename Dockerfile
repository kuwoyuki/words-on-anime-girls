FROM rust:latest as builder
RUN apt update && apt install -y musl-tools musl-dev
RUN update-ca-certificates
WORKDIR /usr/src
# cache deps
RUN USER=root cargo new words_on_anime_girls
COPY Cargo.toml Cargo.lock /usr/src/words_on_anime_girls/
WORKDIR /usr/src/words_on_anime_girls
RUN rustup target add x86_64-unknown-linux-musl
RUN cargo build --target x86_64-unknown-linux-musl --release
# copy src
COPY src /usr/src/words_on_anime_girls/src/
# modify mtime to force cargo to rebuild
RUN touch /usr/src/words_on_anime_girls/src/main.rs

RUN cargo build --target x86_64-unknown-linux-musl --release
RUN strip /usr/src/words_on_anime_girls/target/x86_64-unknown-linux-musl/release/words_on_anime_girls

FROM alpine
COPY --from=builder /usr/src/words_on_anime_girls/target/x86_64-unknown-linux-musl/release/words_on_anime_girls /usr/local/bin
CMD ["/usr/local/bin/words_on_anime_girls"]
