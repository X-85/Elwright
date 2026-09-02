# 多阶段编译：cargo build --release → 静态二进制 → distroless 运行
FROM rust:1.83-bookworm AS builder
WORKDIR /build
# 缓存依赖层
COPY Cargo.toml ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf src target/release/deps/elwright_relay*
COPY src ./src
RUN cargo build --release

FROM gcr.io/distroless/cc-debian12
COPY --from=builder /build/target/release/elwright-relay /usr/local/bin/elwright-relay
ENV BIND_ADDR=0.0.0.0:9000
EXPOSE 9000
USER nonroot
ENTRYPOINT ["/usr/local/bin/elwright-relay"]