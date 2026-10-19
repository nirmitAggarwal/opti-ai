FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        build-essential \
        ca-certificates \
        clang \
        cmake \
        curl \
        git \
        libclang-dev \
        pkg-config \
    && rm -rf /var/lib/apt/lists/*

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:${PATH} \
    LIBCLANG_PATH=/usr/lib/llvm-14/lib

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --default-toolchain 1.85.1 --profile minimal --no-modify-path

WORKDIR /app
COPY . .

RUN cargo test --workspace --offline

CMD ["cargo", "run", "--offline", "-p", "opti-cli", "--", "demo"]
