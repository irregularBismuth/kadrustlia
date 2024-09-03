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

## Useful commands

Delete all containers

```sh
docker rm -f $(docker ps -a -q)
docker system prune -a --volumes
docker build -t kademlia .
docker compose up --build -d
docker ps -a
docker exec -it kadrustlia-node-1 /bin/sh
```

Docker commands

```sh
docker build -t kademlia .
docker compose up --build -d
docker ps -a
docker exec -it kadrustlia-node-1 /bin/sh
```
