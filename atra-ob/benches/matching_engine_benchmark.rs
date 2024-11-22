use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use atra_ob::core::{MatchingEngine, Order, Side, OrderType};
use rust_decimal_macros::dec;
use rust_decimal::Decimal;
use std::time::Instant;
use std::fs::File;

#[derive(serde::Serialize)]
struct BenchmarkResults {
    depth_impact: Vec<DepthMeasurement>,
    latency_distribution: Vec<u128>,
    batch_performance: Vec<BatchMeasurement>,
}

#[derive(serde::Serialize)]
struct DepthMeasurement {
    depth: usize,
    latency_ns: u128,
}

#[derive(serde::Serialize)]
struct BatchMeasurement {
    batch_size: usize,
    total_time_ns: u128,
    orders_per_second: f64,
}

fn run_benchmarks(c: &mut Criterion) {
    let mut results = BenchmarkResults {
        depth_impact: Vec::new(),
        latency_distribution: Vec::new(),
        batch_performance: Vec::new(),
    };

    // Measure depth impact
    for depth in [100, 500, 1000, 5000, 10000].iter() {
        let mut group = c.benchmark_group("depth_impact");
        let mut latencies = Vec::new();

        group.bench_with_input(BenchmarkId::new("match_with_depth", depth), depth, |b, &size| {
            let mut engine = MatchingEngine::new();

            // Pre-fill order book
            for i in 0..size {
                engine.place_order(Order::new(
                    i as u64,
                    dec!(100.0) + Decimal::new((i % 100) as i64, 2),
                    dec!(1.0),
                    Side::Bid,
                    OrderType::Limit
                ));
            }

            b.iter(|| {
                let start = Instant::now();
                let order = Order::new(
                    size as u64 + 1,
                    dec!(100.0),
                    dec!(1.0),
                    Side::Ask,
                    OrderType::Limit
                );
                engine.place_order(order);
                latencies.push(start.elapsed().as_nanos());
            });
        });

        // Record median latency for this depth
        latencies.sort();
        let median_latency = latencies[latencies.len() / 2];
        results.depth_impact.push(DepthMeasurement {
            depth: *depth,
            latency_ns: median_latency,
        });

        group.finish();
    }

    // Measure latency distribution
    let mut engine = MatchingEngine::new();
    let mut latencies = Vec::new();

    // Pre-fill order book
    for i in 0..1000 {
        engine.place_order(Order::new(
            i,
            dec!(100.0) + Decimal::new((i % 100) as i64, 2),
            dec!(1.0),
            Side::Bid,
            OrderType::Limit
        ));
    }

    // Collect latencies for 1000 orders
    for i in 0..1000 {
        let start = Instant::now();
        engine.place_order(Order::new(
            1000 + i,
            dec!(100.0),
            dec!(1.0),
            Side::Ask,
            OrderType::Limit
        ));
        latencies.push(start.elapsed().as_nanos());
    }
    results.latency_distribution = latencies;

    // Measure batch performance
    for batch_size in [100, 1000, 5000].iter() {
        let start = Instant::now();
        let mut engine = MatchingEngine::new();

        for i in 0..*batch_size {
            engine.place_order(Order::new(
                i as u64,
                dec!(100.0),
                dec!(1.0),
                if i % 2 == 0 { Side::Bid } else { Side::Ask },
                OrderType::Limit
            ));
        }

        let elapsed = start.elapsed();
        results.batch_performance.push(BatchMeasurement {
            batch_size: *batch_size,
            total_time_ns: elapsed.as_nanos(),
            orders_per_second: *batch_size as f64 / elapsed.as_secs_f64(),
        });
    }


    let file = File::create("bench_results.json").unwrap();
    serde_json::to_writer_pretty(file, &results).unwrap();
}

criterion_group!(benches, run_benchmarks);
criterion_main!(benches);
