from ubuntu:latest

run apt-get update && apt-get install -y curl build-essential && rm -rf /var/lib/apt/lists/*
run curl https://sh.rustup.rs -sSf | sh -s -- -y
