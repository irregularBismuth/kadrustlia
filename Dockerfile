FROM ubuntu:22.04

RUN apt-get update && apt-get install -y \
    curl \
    build-essential \
    cmake \
    iproute2 \
    iputils-ping && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /usr/src/kadrustlia

COPY . .

RUN cargo build --release

WORKDIR /usr/src/kadrustlia/target/release

CMD ["./kadrustlia"]