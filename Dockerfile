FROM ubuntu:latest

RUN apt update
RUN apt install curl build-essential -y

# Install Rust
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
# RUN . $HOME/.cargo/env

# Install wasm-pack
RUN curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh -s -- -y

# WORKDIR /work
