use crate::core::MatchingEngine;
use crate::core::{Order, OrderType, Side};
use crate::proto;
use crate::proto::order_book_service_server::{
    OrderBookService as GrpcService, OrderBookServiceServer,
};
use crate::proto::{
    CancelOrderRequest, GetOrderBookRequest, GetOrderStatusRequest, GetTradeHistoryRequest, OrderBatchRequest,
    OrderBatchResponse, OrderRequest, OrderResponse, Trade as ProtoTrade, TradeHistoryResponse,
};
use prost_types::Timestamp;
use rust_decimal::Decimal;
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot, Mutex, RwLock};
use tonic::{transport::Server, Request, Response, Status};

#[derive(Clone, Copy)]
pub struct SequencerConfig {
    pub strict_sequence_validation: bool,
}

impl SequencerConfig {
    pub fn from_env() -> Self {
        let strict = std::env::var("ATRA_STRICT_SEQUENCE_VALIDATION")
            .ok()
            .map(|v| matches!(v.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
            .unwrap_or(false);
        Self {
            strict_sequence_validation: strict,
        }
    }
}

struct InstrumentState {
    next_sequence: AtomicU64,
}

impl InstrumentState {
    fn new() -> Self {
        Self {
            next_sequence: AtomicU64::new(1),
        }
    }
}

enum WorkerCommand {
    Place {
        order: Order,
        response: oneshot::Sender<Result<Order, Status>>,
    },
    Cancel {
        order_id: u64,
        response: oneshot::Sender<Result<Order, Status>>,
    },
    Snapshot {
        depth: usize,
        response: oneshot::Sender<Result<proto::OrderBookResponse, Status>>,
    },
    Status {
        order_id: u64,
        response: oneshot::Sender<Result<Order, Status>>,
    },
    Trades {
        limit: usize,
        response: oneshot::Sender<Result<Vec<ProtoTrade>, Status>>,
    },
}

pub struct OrderBookService {
    lanes: Arc<RwLock<HashMap<u32, mpsc::Sender<WorkerCommand>>>>,
    lane_states: Arc<RwLock<HashMap<u32, Arc<InstrumentState>>>>,
    lane_count: u32,
    config: SequencerConfig,
}

impl OrderBookService {
    pub fn new(lane_count: u32, config: SequencerConfig) -> Self {
        Self {
            lanes: Arc::new(RwLock::new(HashMap::new())),
            lane_states: Arc::new(RwLock::new(HashMap::new())),
            lane_count: lane_count.max(1),
            config,
        }
    }

    pub async fn serve(self, addr: &str) -> Result<(), Box<dyn std::error::Error>> {
        let addr = addr.parse()?;
        Server::builder()
            .add_service(OrderBookServiceServer::new(self))
            .serve(addr)
            .await?;
        Ok(())
    }

    fn lane_for_instrument(&self, instrument_id: u32) -> u32 {
        instrument_id % self.lane_count
    }

    async fn lane_sender_for_instrument(&self, instrument_id: u32) -> mpsc::Sender<WorkerCommand> {
        let lane_id = self.lane_for_instrument(instrument_id);
        if let Some(sender) = self.lanes.read().await.get(&lane_id).cloned() {
            return sender;
        }

        let mut lanes = self.lanes.write().await;
        if let Some(sender) = lanes.get(&lane_id).cloned() {
            return sender;
        }

        let (tx, rx) = mpsc::channel(4096);
        tokio::spawn(run_lane_worker(rx));
        lanes.insert(lane_id, tx.clone());
        tx
    }

    async fn state_for_instrument(&self, instrument_id: u32) -> Arc<InstrumentState> {
        if let Some(state) = self.lane_states.read().await.get(&instrument_id).cloned() {
            return state;
        }
        let mut states = self.lane_states.write().await;
        if let Some(state) = states.get(&instrument_id).cloned() {
            return state;
        }
        let state = Arc::new(InstrumentState::new());
        states.insert(instrument_id, state.clone());
        state
    }

    async fn build_engine_order(&self, req: OrderRequest) -> Result<Order, Status> {
        let price =
            Decimal::from_str(&req.price).map_err(|_| Status::invalid_argument("Invalid price format"))?;
        let quantity = Decimal::from_str(&req.quantity)
            .map_err(|_| Status::invalid_argument("Invalid quantity format"))?;
        let state = self.state_for_instrument(req.instrument_id).await;
        let next_expected = state.next_sequence.load(Ordering::SeqCst);
        let sequence = if req.sequence_number == 0 {
            state.next_sequence.fetch_add(1, Ordering::SeqCst)
        } else {
            if self.config.strict_sequence_validation && req.sequence_number != next_expected {
                return Err(Status::failed_precondition(format!(
                    "Out-of-order sequence for instrument {}: expected {}, got {}",
                    req.instrument_id, next_expected, req.sequence_number
                )));
            }
            if req.sequence_number >= next_expected {
                state
                    .next_sequence
                    .store(req.sequence_number.saturating_add(1), Ordering::SeqCst);
            }
            req.sequence_number
        };

        let mut order = Order::new(
            req.id,
            req.instrument_id,
            sequence,
            price,
            quantity,
            if req.side == 0 { Side::Bid } else { Side::Ask },
            if req.order_type == 0 {
                OrderType::Limit
            } else {
                OrderType::Market
            },
        );
        order.ingress_timestamp_ns = req.ingress_timestamp_ns;
        order.idempotency_key = req.idempotency_key;
        Ok(order)
    }
}

async fn run_lane_worker(mut rx: mpsc::Receiver<WorkerCommand>) {
    let engines: Arc<Mutex<HashMap<u32, MatchingEngine>>> = Arc::new(Mutex::new(HashMap::new()));
    let mut seen_idempotency: HashSet<String> = HashSet::new();
    while let Some(cmd) = rx.recv().await {
        match cmd {
            WorkerCommand::Place { order, response } => {
                if let Some(key) = &order.idempotency_key {
                    if seen_idempotency.contains(key) {
                        let _ = response.send(Err(Status::already_exists("Duplicate idempotency key")));
                        continue;
                    }
                    seen_idempotency.insert(key.clone());
                }
                let mut engines_locked = engines.lock().await;
                let engine = engines_locked
                    .entry(order.instrument_id)
                    .or_insert_with(MatchingEngine::new);
                let placed = engine.place_order(order);
                let _ = response.send(Ok(placed));
            }
            WorkerCommand::Cancel { order_id, response } => {
                let mut engines_locked = engines.lock().await;
                let mut cancelled = None;
                for engine in engines_locked.values_mut() {
                    if let Some(order) = engine.cancel_order(order_id) {
                        cancelled = Some(order);
                        break;
                    }
                }
                let _ = response.send(
                    cancelled.ok_or_else(|| Status::not_found("Order not found or cannot be cancelled")),
                );
            }
            WorkerCommand::Snapshot { depth, response } => {
                let mut bids = Vec::new();
                let mut asks = Vec::new();
                let engines_locked = engines.lock().await;
                for engine in engines_locked.values() {
                    let (mut engine_bids, mut engine_asks) = engine.get_order_book(depth);
                    bids.append(&mut engine_bids);
                    asks.append(&mut engine_asks);
                }
                let _ = response.send(Ok(proto::OrderBookResponse {
                    bids: bids
                        .into_iter()
                        .map(|(price, qty)| proto::OrderBookLevel {
                            price: price.to_string(),
                            quantity: qty.to_string(),
                        })
                        .collect(),
                    asks: asks
                        .into_iter()
                        .map(|(price, qty)| proto::OrderBookLevel {
                            price: price.to_string(),
                            quantity: qty.to_string(),
                        })
                        .collect(),
                }));
            }
            WorkerCommand::Status { order_id, response } => {
                let engines_locked = engines.lock().await;
                let mut found = None;
                for engine in engines_locked.values() {
                    if let Some(order) = engine.get_order_status(order_id) {
                        found = Some(order.clone());
                        break;
                    }
                }
                let _ = response.send(found.ok_or_else(|| Status::not_found("Order not found")));
            }
            WorkerCommand::Trades { limit, response } => {
                let engines_locked = engines.lock().await;
                let mut trades = Vec::new();
                for engine in engines_locked.values() {
                    let mut history = engine
                        .get_trade_history(Some(limit))
                        .into_iter()
                        .map(|trade| ProtoTrade {
                            maker_order_id: trade.maker_order_id,
                            taker_order_id: trade.taker_order_id,
                            price: trade.price.to_string(),
                            quantity: trade.quantity.to_string(),
                            side: trade.side as i32,
                            timestamp: trade.timestamp.map(|ts| Timestamp {
                                seconds: ts.timestamp(),
                                nanos: ts.timestamp_subsec_nanos() as i32,
                            }),
                            maker_sequence_number: trade.maker_sequence,
                            taker_sequence_number: trade.taker_sequence,
                            ingress_timestamp_ns: trade.ingress_timestamp_ns,
                        })
                        .collect::<Vec<_>>();
                    trades.append(&mut history);
                }
                let _ = response.send(Ok(trades));
            }
        }
    }
}

#[tonic::async_trait]
impl GrpcService for OrderBookService {
    async fn place_order(
        &self,
        request: Request<OrderRequest>,
    ) -> Result<Response<OrderResponse>, Status> {
        let req = request.into_inner();
        let order = self.build_engine_order(req).await?;
        let lane_sender = self.lane_sender_for_instrument(order.instrument_id).await;
        let (tx, rx) = oneshot::channel();
        lane_sender
            .send(WorkerCommand::Place { order, response: tx })
            .await
            .map_err(|_| Status::internal("Lane worker unavailable"))?;
        let result = rx
            .await
            .map_err(|_| Status::internal("Lane worker response dropped"))??;

        Ok(Response::new(OrderResponse {
            id: result.id,
            price: result.price.to_string(),
            quantity: result.quantity.to_string(),
            remaining_quantity: result.remaining_quantity.to_string(),
            side: result.side as i32,
            order_type: result.order_type as i32,
            status: result.status as i32,
            timestamp: result.timestamp.map(|ts| Timestamp {
                seconds: ts.timestamp(),
                nanos: ts.timestamp_subsec_nanos() as i32,
            }),
            instrument_id: result.instrument_id,
            sequence_number: result.sequence,
            ingress_timestamp_ns: result.ingress_timestamp_ns,
            idempotency_key: result.idempotency_key,
        }))
    }

    // placeholder for now
    async fn place_orders(
	&self,
	request: Request<OrderBatchRequest>,
    ) -> Result<Response<OrderBatchResponse>, Status> {
        let mut responses = Vec::new();
        for order in request.into_inner().orders {
            let placed = self
                .place_order(Request::new(order))
                .await?
                .into_inner();
            responses.push(placed);
        }
        Ok(Response::new(OrderBatchResponse { orders: responses }))
    }
    

    async fn cancel_order(
	&self,
	request: Request<CancelOrderRequest>,
    ) -> Result<Response<OrderResponse>, Status> {
	let order_id = request.into_inner().order_id;
        let mut cancelled_order = None;
        for lane in self.lanes.read().await.values().cloned() {
            let (tx, rx) = oneshot::channel();
            lane.send(WorkerCommand::Cancel {
                order_id,
                response: tx,
            })
            .await
            .map_err(|_| Status::internal("Lane worker unavailable"))?;
            if let Ok(Ok(order)) = rx.await {
                cancelled_order = Some(order);
                break;
            }
        }
	let cancelled_order = cancelled_order.ok_or_else(|| Status::not_found("Order not found or cannot be cancelled"))?;
	Ok(Response::new(OrderResponse {
            id: cancelled_order.id,
            price: cancelled_order.price.to_string(),
            quantity: cancelled_order.quantity.to_string(),
            remaining_quantity: cancelled_order.remaining_quantity.to_string(),
            side: cancelled_order.side as i32,
            order_type: cancelled_order.order_type as i32,
            status: cancelled_order.status as i32,
            timestamp: cancelled_order.timestamp.map(|ts| Timestamp {
                seconds: ts.timestamp(),
                nanos: ts.timestamp_subsec_nanos() as i32,
            }),
            instrument_id: cancelled_order.instrument_id,
            sequence_number: cancelled_order.sequence,
            ingress_timestamp_ns: cancelled_order.ingress_timestamp_ns,
            idempotency_key: cancelled_order.idempotency_key,
        }))
    }

    async fn get_order_book(
        &self,
        request: Request<GetOrderBookRequest>,
    ) -> Result<Response<proto::OrderBookResponse>, Status> {
        let depth = request.into_inner().depth as usize;
        let mut bids = Vec::new();
        let mut asks = Vec::new();
        for lane in self.lanes.read().await.values().cloned() {
            let (tx, rx) = oneshot::channel();
            lane.send(WorkerCommand::Snapshot { depth, response: tx })
                .await
                .map_err(|_| Status::internal("Lane worker unavailable"))?;
            let snapshot = rx
                .await
                .map_err(|_| Status::internal("Lane worker response dropped"))??;
            bids.extend(snapshot.bids);
            asks.extend(snapshot.asks);
        }
        Ok(Response::new(proto::OrderBookResponse { bids, asks }))
    }

    async fn get_order_status(
        &self,
        request: Request<GetOrderStatusRequest>,
    ) -> Result<Response<OrderResponse>, Status> {
        let order_id = request.into_inner().order_id;

        let mut order = None;
        for lane in self.lanes.read().await.values().cloned() {
            let (tx, rx) = oneshot::channel();
            lane.send(WorkerCommand::Status {
                order_id,
                response: tx,
            })
            .await
            .map_err(|_| Status::internal("Lane worker unavailable"))?;
            if let Ok(Ok(found)) = rx.await {
                order = Some(found);
                break;
            }
        }
        let order = order.ok_or_else(|| Status::not_found("Order not found"))?;

        Ok(Response::new(OrderResponse {
            id: order.id,
            price: order.price.to_string(),
            quantity: order.quantity.to_string(),
            remaining_quantity: order.remaining_quantity.to_string(),
            side: order.side as i32,
            order_type: order.order_type as i32,
            status: order.status as i32,
            timestamp: order.timestamp.map(|ts| prost_types::Timestamp {
                seconds: ts.timestamp(),
                nanos: ts.timestamp_subsec_nanos() as i32,
            }),
            instrument_id: order.instrument_id,
            sequence_number: order.sequence,
            ingress_timestamp_ns: order.ingress_timestamp_ns,
            idempotency_key: order.idempotency_key,
        }))
    }


    async fn get_trade_history(&self, request: Request<GetTradeHistoryRequest>) -> Result<Response<TradeHistoryResponse>, Status> {
	let limit = request.into_inner().limit as usize;

        let mut trades = Vec::new();
        for lane in self.lanes.read().await.values().cloned() {
            let (tx, rx) = oneshot::channel();
            lane.send(WorkerCommand::Trades {
                limit,
                response: tx,
            })
            .await
            .map_err(|_| Status::internal("Lane worker unavailable"))?;
            let mut lane_trades = rx
                .await
                .map_err(|_| Status::internal("Lane worker response dropped"))??;
            trades.append(&mut lane_trades);
        }

	Ok(Response::new(TradeHistoryResponse { trades }))
    }
}
