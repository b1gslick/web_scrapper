ARG APP_NAME=web_finder
FROM rust:bookworm as build
ARG APP_NAME
WORKDIR /app

RUN apt-get install perl-base
RUN --mount=type=bind,source=src,target=src \
  --mount=type=bind,source=src/config,target=src/config \
  --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
  --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
  --mount=type=cache,target=/app/target/ \
  --mount=type=cache,target=/usr/local/cargo/registry/ \
  <<EOF
set -e
cargo build --release
cp ./target/release/$APP_NAME /bin/server
EOF

FROM debian:bookworm-slim as final
# FROM hthiemann/docker-chromium-armhf:latest as final
RUN apt-get update \
  && apt-get install libxml2 -y \
  && apt-get install chromium -y \
  && apt-get install -y --reinstall ca-certificates

RUN mkdir /src
COPY src/config src/config
COPY --from=build /bin/server /bin/

CMD ["/bin/server"]
