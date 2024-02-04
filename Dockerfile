FROM rust:bookworm as builder

# 1. Create a new empty shell project
RUN USER=root cargo new --bin web_finder
WORKDIR /web_finder

# 2. Copy our manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# 3. Build only the dependencies to cache them
RUN cargo build --release
RUN rm src/*.rs

# 4. Now that the dependency is built, copy your source code
COPY ./src ./src

# 5. Build for release.
RUN rm ./target/release/deps/web_finder*
RUN cargo build --release

# FROM debian:bookworm-slim
FROM hthiemann/docker-chromium-armhf

# RUN   apt-get update && \ 
# apt install chromium -y

COPY ./src/config ./src/config

COPY --from=builder /web_finder/target/release/web_finder .

CMD ["./web_finder"]
