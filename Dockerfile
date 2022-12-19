FROM jdrouet/wasm-pack:latest as builder
COPY . .
RUN rustup component add rust-src --toolchain nightly-2022-06-01-x86_64-unknown-linux-gnu
CMD ["wasm-pack", "build", "--target", "web", "-d", "wasm/pkg"]

# FROM debian:buster-slim
# RUN apt-get update && apt-get install -y extra-runtime-dependencies && rm -rf /var/lib/apt/lists/*
# COPY --from=builder /usr/local/cargo/bin/myapp /usr/local/bin/myapp
# CMD ["myapp"]