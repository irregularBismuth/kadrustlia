# Install rust

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

# Docker

## install docker

TODO:

## Building the image

```sh
docker build -t kademlia .
```

## Running the image

```sh
docker run -it kadrustlia
```

## Running locally

```sh
cargo run --release
```

## Useful commands

Delete all containers

```sh
docker rm -f $(docker ps -a -q)
docker system prune -a --volumes
```

Docker commands

```sh
docker build -t kademlia .
docker compose up --build -d
docker ps -a
docker exec -it kadrustlia-kademliaNodes-1 /bin/sh
```

