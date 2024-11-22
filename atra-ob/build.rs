fn main() {
    tonic_build::compile_protos("../atra-proto/proto/orderbook.proto")
        .unwrap_or_else(|e| panic!("Failed to compile protos {:?}", e));
}
