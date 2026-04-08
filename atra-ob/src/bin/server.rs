use atra_ob::api::service::OrderBookService;
use atra_ob::api::service::SequencerConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let lane_count = std::env::var("ATRA_LANE_COUNT")
        .ok()
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(4);
    let service = OrderBookService::new(lane_count, SequencerConfig::from_env());

    println!("Starting order book server on 0.0.0.0:50051");
    service.serve("0.0.0.0:50051").await
}
