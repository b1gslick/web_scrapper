ARG RUST_VERSION=1.74.1
ARG APP_NAME=web_scraper
FROM rust:${RUST_VERSION}-slim-bullseye AS build
ARG APP_NAME
WORKDIR /app

RUN --mount=type=bind,source=src,target=src \
  --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
  --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
  --mount=type=cache,target=/app/target/ \
  --mount=type=cache,target=/usr/local/cargo/registry/ \
  <<EOF
set -e
cargo build --locked --release
cp ./target/release/$APP_NAME /bin/server
EOF

FROM hthiemann/docker-chromium-armhf:latest as final

# create simple user
# ARG UID=10001
# RUN adduser --disabled-password --gecos '' newuser --uid "${UID}"

# USER newuser

# copy binaries
COPY --from=build /bin/server /bin/

CMD ["/bin/server"]
