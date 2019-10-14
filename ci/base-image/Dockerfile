FROM rust:1.38.0-slim-buster

# Install additional packages
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        cmake \
        libclang-dev \
        llvm-dev \
        clang \
        g++ \
        libssl-dev \
        pkg-config \
        curl \
    && apt-get autoremove \
    && rm -rf /var/lib/apt/lists/*

RUN curl -sSLO https://github.com/mozilla/sccache/releases/download/0.2.12/sccache-0.2.12-x86_64-unknown-linux-musl.tar.gz \
    && echo "26fd04c1273952cc2a0f359a71c8a1857137f0ee3634058b3f4a63b69fc8eb7f  sccache-0.2.12-x86_64-unknown-linux-musl.tar.gz" > sccache.sum \
    && sha256sum -c sccache.sum \
    && tar -zxf sccache-0.2.12-x86_64-unknown-linux-musl.tar.gz \
    && mv sccache-0.2.12-x86_64-unknown-linux-musl/sccache /usr/local/bin/sccache \
    && rm -rf sccache-0.2.12-x86_64-unknown-linux-musl

# Use correct toolchain, install WASM components required by the ledger, and
# Rust utilities necessary in some CI pipeline steps i.e. formatting check,
# linting.
RUN rustup default nightly-2019-09-05 \
    && rustup update \
    && rustup component add rustfmt \
    && rustup component add clippy \
    && rustup target add wasm32-unknown-unknown \
    && command -v wasm-gc || cargo install --git https://github.com/alexcrichton/wasm-gc

# Setup defaults for caching
VOLUME /cache
ENV SCCACHE_DIR=/cache/sccache RUSTC_WRAPPER=sccache CARGO_HOME=/cache/cargo
