pub mod core;
pub mod api;
pub mod transport;

pub mod proto {
    tonic::include_proto!("orderbook");
}
