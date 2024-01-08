
ARG RUST_VERSION=1.74.1
# ARG TARGET=x86_64-unknown-linux-musl
FROM rust:${RUST_VERSION}-slim-bullseye AS build
ARG TARGET

WORKDIR /app

RUN apt-get update && \
  apt-get install -y  pkg-config cmake make g++ libssl-dev musl-tools musl-dev build-essential gcc-x86-64-linux-gnu
# rustup target add ${TARGET}

ENV RUSTFLAGS='-C linker=x86_64-linux-gnu-gcc'
ENV CC='gcc'
ENV CC_x86_64_unknown_linux_musl=x86_64-linux-gnu-gcc
ENV CC_x86_64-unknown-linux-musl=x86_64-linux-gnu-gcc

RUN --mount=type=bind,source=src,target=src \
  --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
  --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
  --mount=type=cache,target=/app/target/ \
  --mount=type=cache,target=/usr/local/cargo/registry/ \
  <<EOF
set -e
cargo build --locked --release 
cp ./target/release/server /bin/server
EOF

FROM debian:bullseye-slim AS final

RUN apt-get update; apt-get clean && \
  apt-get install -y wget && \
  apt install -y openssl && \
  apt-get install -y gnupg && \
  apt-get install -y gcc
# Set the Chrome repo.
RUN wget -q -O - https://dl-ssl.google.com/linux/linux_signing_key.pub | apt-key add - \
  && echo "deb http://dl.google.com/linux/chrome/deb/ stable main" >> /etc/apt/sources.list.d/google.list

# Install Chrome.
RUN apt-get update && apt-get -y install google-chrome-stable

ARG UID=10001

RUN adduser \
  --disabled-password \
  --gecos "" \
  --home "/nonexistent" \
  --shell "/sbin/nologin" \
  --no-create-home \
  --uid "${UID}" \
  appuser

USER appuser


COPY --from=build /bin/server .
COPY ./src/config ./src/config

CMD ["./server"]
