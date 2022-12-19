FROM jdrouet/wasm-pack as builder
WORKDIR /
COPY . .
CMD ["wasm-pack", "build", "--target", "web", "-d", "wasm/pkg"]

# FROM debian:buster-slim
# RUN apt-get update && apt-get install -y extra-runtime-dependencies && rm -rf /var/lib/apt/lists/*
# COPY --from=builder /usr/local/cargo/bin/myapp /usr/local/bin/myapp
# CMD ["myapp"]