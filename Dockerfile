# For local builds only (not for CI).

FROM rust:1.72.1 as builder
WORKDIR /app

# Create a dummy project and build the app's dependencies.
# If the Cargo.toml or Cargo.lock files have not changed,
# we can use the docker build cache and skip these (typically slow) steps.
RUN USER=root cargo init .
COPY .cargo .cargo
COPY Cargo.toml Cargo.lock ./
RUN cargo build

# Remove files with newer timestamps.
RUN rm src/*.rs

COPY src src
RUN cargo build

FROM debian:stable-slim
RUN groupadd scrapper && useradd -m -d /app -g scrapper scrapper
USER scrapper
WORKDIR /app
COPY --chown=scrapper:scrapper --from=builder /app/target/debug/web_finder /app/
ENTRYPOINT [ "/app/web_finder"]
