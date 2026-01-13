FROM ghcr.io/rust-cross/cargo-zigbuild:0.20 AS chef
USER root
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY Cargo.toml Cargo.lock .
RUN mkdir src
RUN echo 'fn main(){}' > src/main.rs
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
