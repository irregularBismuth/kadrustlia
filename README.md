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
```
