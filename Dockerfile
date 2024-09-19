FROM rust:latest AS cargo-build

RUN apt-get update && apt-get install -y --no-install-recommends \
    musl-tools \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /usr/src/kadrustlia

COPY . .

RUN cargo build --release --target x86_64-unknown-linux-musl

FROM alpine:latest

WORKDIR /home/kadrustlia/bin/

RUN apk update && apk add --no-cache file iproute2 iputils-ping 

COPY --from=cargo-build /usr/src/kadrustlia/target/x86_64-unknown-linux-musl/release/kadrustlia .

CMD ["./kadrustlia"]
