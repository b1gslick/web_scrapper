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


FROM debian:bookworm-slim

ARG TOKEN
ARG URLS
ARG KW

ENV TOKEN=$TOKEN
ENV URLS=$URLS
ENV KW=$KW

RUN apt-get update; apt-get clean

# Install wget.
RUN apt-get install -y wget
RUN apt install -y openssl
RUN apt-get install -y gnupg
RUN apt-get install -y gcc
# Set the Chrome repo.
RUN wget -q -O - https://dl-ssl.google.com/linux/linux_signing_key.pub | apt-key add - \
    && echo "deb http://dl.google.com/linux/chrome/deb/ stable main" >> /etc/apt/sources.list.d/google.list

# Install Chrome.
RUN apt-get update && apt-get -y install google-chrome-stable

COPY --from=builder /web_finder/target/release/web_finder .

CMD ["./web_finder"]