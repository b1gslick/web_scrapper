FROM rust:1.72.1 

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
RUN cargo install --path .

CMD ["web_finder"]
