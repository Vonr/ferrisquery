FROM rust:1-alpine AS chef
USER root

# Install bash and curl
RUN apk add --no-cache \
    bash \
    curl

# Install cargo-binstall
RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash

# Install cargo-chef
RUN cargo binstall cargo-chef

# Install zig
ARG ZIG_VERSION=0.13.0
RUN curl -L "https://ziglang.org/download/${ZIG_VERSION}/zig-linux-$(uname -m)-${ZIG_VERSION}.tar.xz" | tar -xJC /usr/local && \
    ln -s "/usr/local/zig-linux-$(uname -m)-${ZIG_VERSION}/zig" /usr/local/bin/zig

# Install cargo-zigbuild
ARG ZIGBUILD_VERSION=0.21.1
RUN curl -L "https://github.com/rust-cross/cargo-zigbuild/releases/download/v${ZIGBUILD_VERSION}/cargo-zigbuild-v${ZIGBUILD_VERSION}.$(uname -m)-unknown-linux-musl.tar.gz" | tar -xzC /usr/local/bin

WORKDIR /app

FROM chef AS planner
RUN mkdir src
RUN echo 'fn main(){}' > src/main.rs
COPY Cargo.toml Cargo.lock .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json --zigbuild
COPY Cargo.toml Cargo.lock .
COPY src/ src/
RUN cargo zigbuild --release --target x86_64-unknown-linux-musl

FROM scratch AS runner
WORKDIR /app
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/ferrisquery /app/
CMD ["/app/ferrisquery"]
