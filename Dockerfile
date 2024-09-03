from ubuntu:latest as builder 

run apt-get update && apt-get install -y iproute2 iputils-ping  curl build-essential && rm -rf /var/lib/apt/lists/*
run curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
workdir /src/app
copy Cargo.toml ./
copy src/ src/
run cargo build --release

CMD ["./target/release/kadrustlia"]
