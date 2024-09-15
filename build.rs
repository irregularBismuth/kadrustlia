fn main() {
    tonic_build::compile_protos("proto/kademlia.proto").unwrap();
}