# Image to run builds of the project
#
# Includes
#
# * Rustup with `nightly-2019-09-05` toolchain (see RUST_VERSION)
# * Additional rustup components and the wasm32 target
# * wasm-gc
# * sccache v0.2.12

FROM debian:buster-slim

SHELL ["/bin/bash", "-c"]

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH \
    RUST_VERSION=nightly-2019-09-05 \
    SCCACHE_DIR=/cache/sccache \
    RUSTC_WRAPPER=sccache \
    CARGO_HOME=/usr/local/cargo

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates \
        clang \
        cmake \
        gcc \
        libclang-dev \
        libssl-dev \
        llvm-dev \
        make \
        pkg-config \
        wget \
    && apt-get autoremove \
    && rm -rf /var/lib/apt/lists/*

# Install sccache
RUN set -euxo pipefail; \
    sccache_version=0.2.12; \
    sccache_base=sccache-$sccache_version-x86_64-unknown-linux-musl; \
    wget --quiet https://github.com/mozilla/sccache/releases/download/$sccache_version/$sccache_base.tar.gz; \
    echo "26fd04c1273952cc2a0f359a71c8a1857137f0ee3634058b3f4a63b69fc8eb7f  $sccache_base.tar.gz" | sha256sum -c -; \
    tar -zxf "$sccache_base.tar.gz"; \
    mv "$sccache_base/sccache" /usr/local/bin/sccache; \
    rm -r "$sccache_base.tar.gz" "$sccache_base"

# Install rustup and default toolchain from RUST_VERSION. This is copied from
# https://github.com/rust-lang/docker-rust/blob/1d8ef2981548b4b54767e09274c26b2dd6a4e9ec/1.38.0/buster/slim/Dockerfile
RUN set -euxo pipefail; \
    dpkgArch="$(dpkg --print-architecture)"; \
    case "${dpkgArch##*-}" in \
        amd64) rustArch='x86_64-unknown-linux-gnu'; rustupSha256='36285482ae5c255f2decfab27d32ba19465804cb3ddf5a23e6ff2a7b0f6e0250' ;; \
        armhf) rustArch='armv7-unknown-linux-gnueabihf'; rustupSha256='cb20e54566d4b13434dea1776a961cf7f97afcc292cb4b0fec533503dd2434d0' ;; \
        arm64) rustArch='aarch64-unknown-linux-gnu'; rustupSha256='58e19ae12101103ccc50b04a2579b9238163f87a27da5078cefc900098f257ab' ;; \
        i386) rustArch='i686-unknown-linux-gnu'; rustupSha256='d3c42fb8b25f87eb049b6177611eea7d4fd51273de4113706f43cccf5610cfc7' ;; \
        *) echo >&2 "unsupported architecture: ${dpkgArch}"; exit 1 ;; \
    esac; \
    url="https://static.rust-lang.org/rustup/archive/1.19.0/${rustArch}/rustup-init"; \
    wget --quiet "$url"; \
    echo "${rustupSha256} *rustup-init" | sha256sum -c -; \
    chmod +x rustup-init; \
    ./rustup-init -y --no-modify-path --default-toolchain $RUST_VERSION; \
    rm rustup-init; \
    chmod -R a+w $RUSTUP_HOME $CARGO_HOME; \
    rustup --version; \
    cargo --version; \
    rustc --version;

# Add components to toolchain install wasm-gc
RUN rustup component add rustfmt \
    && rustup component add clippy \
    && rustup target add wasm32-unknown-unknown \
    && cargo install --git https://github.com/alexcrichton/wasm-gc

VOLUME /cache
ENV CARGO_HOME=/cache/cargo
