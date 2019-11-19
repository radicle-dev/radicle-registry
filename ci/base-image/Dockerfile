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
    RUST_VERSION=nightly-2019-11-17 \
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
# https://github.com/rust-lang/docker-rust/blob/8d0f25416858e2c1f59511a15c2bd0445b402caa/1.39.0/buster/slim/Dockerfile
RUN set -eux; \
    dpkgArch="$(dpkg --print-architecture)"; \
    case "${dpkgArch##*-}" in \
        amd64) rustArch='x86_64-unknown-linux-gnu'; rustupSha256='e68f193542c68ce83c449809d2cad262cc2bbb99640eb47c58fc1dc58cc30add' ;; \
        armhf) rustArch='armv7-unknown-linux-gnueabihf'; rustupSha256='7c1c329a676e50c287d8183b88f30cd6afd0be140826a9fbbc0e3d717fab34d7' ;; \
        arm64) rustArch='aarch64-unknown-linux-gnu'; rustupSha256='d861cc86594776414de001b96964be645c4bfa27024052704f0976dc3aed1b18' ;; \
        i386) rustArch='i686-unknown-linux-gnu'; rustupSha256='89f1f797dca2e5c1d75790c3c6b7be0ee473a7f4eca9663e623a41272a358da0' ;; \
        *) echo >&2 "unsupported architecture: ${dpkgArch}"; exit 1 ;; \
    esac; \
    url="https://static.rust-lang.org/rustup/archive/1.20.2/${rustArch}/rustup-init"; \
    wget "$url"; \
    echo "${rustupSha256} *rustup-init" | sha256sum -c -; \
    chmod +x rustup-init; \
    ./rustup-init -y --no-modify-path --profile minimal --default-toolchain $RUST_VERSION; \
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
