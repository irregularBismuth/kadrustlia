FROM ubuntu:22.04 as builder

RUN apt-get update && apt-get install -y \
    curl \
    build-essential \
    iproute2 \
    iputils-ping && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /src/app

COPY . .

RUN cargo build --release

CMD ["./target/release/kadrustlia"]
