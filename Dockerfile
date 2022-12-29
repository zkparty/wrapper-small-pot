FROM ubuntu:22.04

RUN apt-get update
RUN apt-get install -y \
     curl \
     build-essential \
     clang-3.8

# Install Rust
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
RUN rustup target add wasm32-unknown-unknown
RUN rustup toolchain install nightly-2022-06-01-x86_64-unknown-linux-gnu
RUN rustup component add rust-src --toolchain nightly-2022-06-01-x86_64-unknown-linux-gnu

# Install wasm-pack
RUN curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh -s -- -y

COPY . /root

WORKDIR /root

RUN echo '#!/bin/bash\nwasm-pack build --target web -d wasm/pkg' >> build.sh
RUN chmod +x build.sh

CMD ["./build.sh"]
