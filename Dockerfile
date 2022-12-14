FROM ubuntu:22.04

COPY . /root

# Install Rust and prerequisites
RUN apt-get update && \
    apt-get install -y \
    curl \
    build-essential \
    clang-3.8 && \
    curl https://sh.rustup.rs -sSf | sh -s -- -y && \
    chmod +x /root/build.sh

ENV PATH="/root/.cargo/bin:${PATH}"
# Install Rust components and wasm-pack
RUN rustup target add wasm32-unknown-unknown && \
    rustup toolchain install nightly-2022-06-01-x86_64-unknown-linux-gnu && \
    rustup component add rust-src --toolchain nightly-2022-06-01-x86_64-unknown-linux-gnu && \
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh -s -- -y

WORKDIR /root

CMD ["./build.sh"]
