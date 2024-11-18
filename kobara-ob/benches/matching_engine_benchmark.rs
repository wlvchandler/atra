use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use kobara_ob::core::{MatchingEngine, Order, Side, OrderType};
use rust_decimal_macros::dec;
use rust_decimal::Decimal;
use rust_decimal::prelude::*;  // Add FromPrimitive trait
use rand::Rng;
use std::time::{Duration, Instant};

// Helper to create orders with random prices around a reference
fn generate_random_orders(count: usize, is_buy: bool) -> Vec<Order> {
    let mut rng = rand::thread_rng();
    let base_price = dec!(100.0);
    let mut orders = Vec::with_capacity(count);

    for i in 0..count {
        // Generate price variation of ±5 with 2 decimal places
        let price_offset = (rng.gen_range(-500..500) as i64) / 100;
        let price = base_price + Decimal::new(price_offset, 2);

        // Generate quantity between 1 and 10 with 2 decimal places
        let quantity = Decimal::new(rng.gen_range(100..1000), 2);

        orders.push(Order::new(
            i as u64,
            price,
            quantity,
            if is_buy { Side::Bid } else { Side::Ask },
            OrderType::Limit
        ));
    }
    orders
}

// Benchmark single order placement
fn bench_single_order(c: &mut Criterion) {
    let mut group = c.benchmark_group("single_order");
    group.measurement_time(Duration::from_secs(10));

    let mut engine = MatchingEngine::new();

    // Pre-fill order book
    for order in generate_random_orders(1000, true) {
        engine.place_order(order);
    }

    group.bench_function("place_matching_order", |b| {
        b.iter_with_setup(
            || Order::new(10001, dec!(100.0), dec!(1.0), Side::Ask, OrderType::Limit),
            |order| {
                black_box(engine.place_order(order));
            }
        )
    });

    group.finish();
}

// Benchmark order book filled with various depths
fn bench_orderbook_depth(c: &mut Criterion) {
    let mut group = c.benchmark_group("orderbook_depth");
    group.measurement_time(Duration::from_secs(10));

    for depth in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::new("matching_order", depth), depth, |b, &depth| {
            let mut engine = MatchingEngine::new();

            // Pre-fill order book
            for order in generate_random_orders(depth, true) {
                engine.place_order(order);
            }

            b.iter_with_setup(
                || Order::new(depth as u64 + 1, dec!(100.0), dec!(1.0), Side::Ask, OrderType::Limit),
                |order| {
                    black_box(engine.place_order(order));
                }
            )
        });
    }

    group.finish();
}

// Benchmark continuous matching with high throughput
fn bench_continuous_matching(c: &mut Criterion) {
    let mut group = c.benchmark_group("continuous_matching");
    group.measurement_time(Duration::from_secs(10));

    for batch_size in [100, 1000].iter() {
        group.bench_with_input(BenchmarkId::new("order_batch", batch_size), batch_size, |b, &size| {
            b.iter_with_setup(
                || {
                    let engine = MatchingEngine::new();
                    let buy_orders = generate_random_orders(size, true);
                    let sell_orders = generate_random_orders(size, false);
                    (engine, buy_orders, sell_orders)
                },
                |(mut engine, buy_orders, sell_orders)| {
                    // Interleave buy and sell orders to create matches
                    for i in 0..size {
                        black_box(engine.place_order(buy_orders[i].clone()));
                        black_box(engine.place_order(sell_orders[i].clone()));
                    }
                }
            )
        });
    }

    group.finish();
}

// Measure latency distribution
fn measure_latency_distribution() -> Vec<Duration> {
    let mut engine = MatchingEngine::new();
    let mut latencies = Vec::with_capacity(1000);

    // Pre-fill order book
    for order in generate_random_orders(1000, true) {
        engine.place_order(order);
    }

    // Measure individual order placement latencies
    for order in generate_random_orders(1000, false) {
        let start = Instant::now();
        engine.place_order(order);
        latencies.push(start.elapsed());
    }

    latencies
}

fn analyze_latency_distribution(c: &mut Criterion) {
    let mut group = c.benchmark_group("latency_distribution");

    let id = "measure_latencies";
    group.bench_function(id, |b| {
        b.iter_with_setup(
            || {
                // run latency measurement
                measure_latency_distribution()
            },
            |latencies| {
                // calculate percentiles
                let mut sorted_latencies = latencies.clone();
                sorted_latencies.sort();

                let p50 = sorted_latencies[latencies.len() / 2];
                let p95 = sorted_latencies[(latencies.len() as f64 * 0.95) as usize];
                let p99 = sorted_latencies[(latencies.len() as f64 * 0.99) as usize];

                black_box((p50, p95, p99))
            }
        );
    });

    let sample_latencies = measure_latency_distribution();
    let mut sorted_latencies = sample_latencies.clone();
    sorted_latencies.sort();

    println!("\nFinal Latency Distribution Summary:");
    println!("p50: {:?}", sorted_latencies[sample_latencies.len() / 2]);
    println!("p95: {:?}", sorted_latencies[(sample_latencies.len() as f64 * 0.95) as usize]);
    println!("p99: {:?}", sorted_latencies[(sample_latencies.len() as f64 * 0.99) as usize]);

    group.finish();
}

criterion_group!{
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = bench_single_order, bench_orderbook_depth, bench_continuous_matching, analyze_latency_distribution
}
criterion_main!(benches);
