use kobara_ob::api::service::OrderBookService;
use kobara_ob::core::MatchingEngine;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let meng = MatchingEngine::new();
    let service = OrderBookService::new(meng);

    println!("Starting order book server on 127.0.0.1:50051");
    service.serve("127.0.0.1:50051").await
}
