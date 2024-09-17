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
```

Docker commands

```sh
docker build -t kademlia .
docker compose up --build -d
docker ps -a
docker exec -it kadrustlia-kademliaNodes-1 /bin/sh
```

## Send rpc:
### docker exec
to self:
```sh
grpcurl -plaintext -import-path /proto -proto kademlia.proto -d '{"contact_id": "123"}' 0.0.0.0:50051 kademlia.Kademlia/LookupContact
```

to another recipient
```sh
grpcurl -plaintext -import-path /proto -proto kademlia.proto -d '{"contact_id": "123"}' 172.18.X.X:50051 kademlia.Kademlia/LookupContact
```

### Local testing:
#### Prerequisites:
- protoc: https://github.com/protocolbuffers/protobuf/releases
- grpcurl: https://github.com/fullstorydev/grpcurl/releases

Also use these in [main](src/main.rs):
```rust
let addr: SocketAddr = "[::1]:50051".parse()?;
let client_url = format!("http://{}", addr);
```

#### Send rpc:
```sh
grpcurl -plaintext -proto ./proto/kademlia.proto -d '{"contact_id": "123"}' [::1]:50051 kademlia.Kademlia/LookupContact

grpcurl -plaintext -proto ./proto/kademlia.proto -d '{"hash": "example_hash"}' [::1]:50051 kademlia.Kademlia/LookupData

grpcurl -plaintext -proto ./proto/kademlia.proto -d '{"data": "VGhpcyBpcyBhIHRlc3Q="}' [::1]:50051 kademlia.Kademlia/Store
```
