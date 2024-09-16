FROM rust:latest as cargo-build

RUN apt-get update && apt-get install -y --no-install-recommends \
    protobuf-compiler \
    musl-tools \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /usr/src/kadrustlia

COPY . .

RUN cargo build --release --target x86_64-unknown-linux-musl

FROM alpine:latest

WORKDIR /home/kadrustlia/bin/

RUN apk add --no-cache file iproute2 iputils-ping
RUN apk update && apk add --no-cache --repository=http://dl-cdn.alpinelinux.org/alpine/edge/testing grpcurl

COPY --from=cargo-build /usr/src/kadrustlia/target/x86_64-unknown-linux-musl/release/kadrustlia .

COPY ./proto /proto

CMD ["./kadrustlia"]
