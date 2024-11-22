FROM rust:1.75-slim as builder

WORKDIR /usr/src/app

RUN apt-get update && apt-get install -y protobuf-compiler

COPY atra-proto/proto /usr/src/app/atra-proto/proto

COPY atra-ob /usr/src/app/atra-ob

WORKDIR /usr/src/app/atra-ob
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    libssl-dev \
    ca-certificates \
    netcat-traditional \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/atra-ob/target/release/server /usr/local/bin/

EXPOSE 50051
ENV BIND_ADDRESS=0.0.0.0:50051

CMD ["server"]
