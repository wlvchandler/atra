use kobara_ob::core::OrderBook;
use kobara_ob::api::service::OrderBookService;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let book = OrderBook::new();
    let service = OrderBookService::new(book);

    println!("Starting order book server on 127.0.0.1:50051");
    service.serve("127.0.0.1:50051").await
}
