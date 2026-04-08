use crate::core::MatchingEngine;
use crate::core::{Order, OrderType, Side};
use crate::proto;
use crate::proto::order_book_service_server::{
    OrderBookService as GrpcService, OrderBookServiceServer,
};
use crate::proto::{
    BatchMode, CancelOrderBatchRequest, CancelOrderBatchResponse, CancelOrderRequest, DecimalValue, ErrorCode,
    ErrorDetail, GetOrderBookRequest, GetOrderStatusRequest, GetTradeHistoryRequest, OrderBatchItemResult,
    OrderBatchRequest, OrderBatchResponse, OrderRequest, OrderResponse, Side as ProtoSide,
    StreamOrderBookRequest, StreamTradeHistoryRequest, Trade as ProtoTrade, TradeHistoryResponse,
};
use prost_types::Timestamp;
use rust_decimal::Decimal;
use std::collections::{HashMap, HashSet};
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use futures::Stream;
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
        instrument_id: u32,
        idempotency_key: Option<String>,
        response: oneshot::Sender<Result<Order, Status>>,
    },
    Snapshot {
        depth: usize,
        instrument_id: u32,
        response: oneshot::Sender<Result<proto::OrderBookResponse, Status>>,
    },
    Status {
        order_id: u64,
        instrument_id: u32,
        response: oneshot::Sender<Result<Order, Status>>,
    },
    Trades {
        limit: usize,
        instrument_id: u32,
        response: oneshot::Sender<Result<Vec<ProtoTrade>, Status>>,
    },
}

#[derive(Clone)]
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
        let price = decimal_from_proto(req.price.as_ref(), "price")?;
        let quantity = decimal_from_proto(req.quantity.as_ref(), "quantity")?;
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

        let side = match ProtoSide::try_from(req.side) {
            Ok(ProtoSide::Bid) => Side::Bid,
            Ok(ProtoSide::Ask) => Side::Ask,
            _ => return Err(Status::invalid_argument("Invalid side")),
        };
        let order_type = match proto::OrderType::try_from(req.order_type) {
            Ok(proto::OrderType::Limit) => OrderType::Limit,
            Ok(proto::OrderType::Market) => OrderType::Market,
            _ => return Err(Status::invalid_argument("Invalid order_type")),
        };

        let mut order = Order::new(
            req.id,
            req.instrument_id,
            sequence,
            price,
            quantity,
            side,
            order_type,
        );
        order.ingress_timestamp_ns = req.ingress_timestamp_ns;
        order.idempotency_key = req.idempotency_key;
        Ok(order)
    }
}

fn decimal_from_proto(value: Option<&DecimalValue>, field_name: &str) -> Result<Decimal, Status> {
    let value = value.ok_or_else(|| Status::invalid_argument(format!("Missing {field_name}")))?;
    if value.scale < 0 {
        return Err(Status::invalid_argument(format!(
            "Invalid {field_name} scale: {}",
            value.scale
        )));
    }
    let scale = value.scale as u32;
    Ok(Decimal::from_i128_with_scale(value.units as i128, scale))
}

fn decimal_to_proto(value: Decimal) -> DecimalValue {
    DecimalValue {
        units: value.mantissa() as i64,
        scale: value.scale() as i32,
    }
}

fn status_from_error(err: &Status) -> ErrorDetail {
    let code = match err.code() {
        tonic::Code::InvalidArgument => ErrorCode::InvalidArgument,
        tonic::Code::NotFound => ErrorCode::NotFound,
        tonic::Code::FailedPrecondition => ErrorCode::FailedPrecondition,
        tonic::Code::AlreadyExists => ErrorCode::AlreadyExists,
        _ => ErrorCode::Internal,
    };
    ErrorDetail {
        code: code as i32,
        message: err.message().to_string(),
    }
}

fn order_to_response(result: Order) -> OrderResponse {
    let side = match result.side {
        Side::Bid => ProtoSide::Bid as i32,
        Side::Ask => ProtoSide::Ask as i32,
    };
    let order_type = match result.order_type {
        OrderType::Limit => proto::OrderType::Limit as i32,
        OrderType::Market => proto::OrderType::Market as i32,
    };
    let status = match result.status {
        crate::core::OrderStatus::Pending => proto::OrderStatus::Pending as i32,
        crate::core::OrderStatus::PartiallyFilled => proto::OrderStatus::PartiallyFilled as i32,
        crate::core::OrderStatus::Filled => proto::OrderStatus::Filled as i32,
        crate::core::OrderStatus::Cancelled => proto::OrderStatus::Cancelled as i32,
    };

    OrderResponse {
        id: result.id,
        price: Some(decimal_to_proto(result.price)),
        quantity: Some(decimal_to_proto(result.quantity)),
        remaining_quantity: Some(decimal_to_proto(result.remaining_quantity)),
        side,
        order_type,
        status,
        timestamp: result.timestamp.map(|ts| Timestamp {
            seconds: ts.timestamp(),
            nanos: ts.timestamp_subsec_nanos() as i32,
        }),
        instrument_id: result.instrument_id,
        sequence_number: result.sequence,
        ingress_timestamp_ns: result.ingress_timestamp_ns,
        idempotency_key: result.idempotency_key,
    }
}

async fn run_lane_worker(mut rx: mpsc::Receiver<WorkerCommand>) {
    let engines: Arc<Mutex<HashMap<u32, MatchingEngine>>> = Arc::new(Mutex::new(HashMap::new()));
    let mut seen_idempotency: HashSet<String> = HashSet::new();
    let mut cancel_idempotency_results: HashMap<String, Order> = HashMap::new();
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
            WorkerCommand::Cancel {
                order_id,
                instrument_id,
                idempotency_key,
                response,
            } => {
                if let Some(key) = &idempotency_key {
                    if let Some(existing) = cancel_idempotency_results.get(key).cloned() {
                        let _ = response.send(Ok(existing));
                        continue;
                    }
                }
                let mut engines_locked = engines.lock().await;
                let cancelled = engines_locked
                    .get_mut(&instrument_id)
                    .and_then(|engine| engine.cancel_order(order_id));
                let result =
                    cancelled.ok_or_else(|| Status::not_found("Order not found or cannot be cancelled"));
                if let (Ok(order), Some(key)) = (&result, idempotency_key) {
                    cancel_idempotency_results.insert(key, order.clone());
                }
                let _ = response.send(result);
            }
            WorkerCommand::Snapshot {
                depth,
                instrument_id,
                response,
            } => {
                let mut bids = Vec::new();
                let mut asks = Vec::new();
                let engines_locked = engines.lock().await;
                if let Some(engine) = engines_locked.get(&instrument_id) {
                    let (mut engine_bids, mut engine_asks) = engine.get_order_book(depth);
                    bids.append(&mut engine_bids);
                    asks.append(&mut engine_asks);
                }
                let _ = response.send(Ok(proto::OrderBookResponse {
                    bids: bids
                        .into_iter()
                        .map(|(price, qty)| proto::OrderBookLevel {
                            price: Some(decimal_to_proto(price)),
                            quantity: Some(decimal_to_proto(qty)),
                        })
                        .collect(),
                    asks: asks
                        .into_iter()
                        .map(|(price, qty)| proto::OrderBookLevel {
                            price: Some(decimal_to_proto(price)),
                            quantity: Some(decimal_to_proto(qty)),
                        })
                        .collect(),
                }));
            }
            WorkerCommand::Status {
                order_id,
                instrument_id,
                response,
            } => {
                let engines_locked = engines.lock().await;
                let found = engines_locked
                    .get(&instrument_id)
                    .and_then(|engine| engine.get_order_status(order_id).cloned());
                let _ = response.send(found.ok_or_else(|| Status::not_found("Order not found")));
            }
            WorkerCommand::Trades {
                limit,
                instrument_id,
                response,
            } => {
                let engines_locked = engines.lock().await;
                let mut trades = Vec::new();
                if let Some(engine) = engines_locked.get(&instrument_id) {
                    let mut history = engine
                        .get_trade_history(Some(limit))
                        .into_iter()
                        .map(|trade| ProtoTrade {
                            maker_order_id: trade.maker_order_id,
                            taker_order_id: trade.taker_order_id,
                            price: Some(decimal_to_proto(trade.price)),
                            quantity: Some(decimal_to_proto(trade.quantity)),
                            side: match trade.side {
                                Side::Bid => ProtoSide::Bid as i32,
                                Side::Ask => ProtoSide::Ask as i32,
                            },
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
    type StreamOrderBookStream = Pin<Box<dyn Stream<Item = Result<proto::OrderBookResponse, Status>> + Send>>;
    type StreamTradeHistoryStream = Pin<Box<dyn Stream<Item = Result<ProtoTrade, Status>> + Send>>;

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

        Ok(Response::new(order_to_response(result)))
    }

    // placeholder for now
    async fn place_orders(
	&self,
	request: Request<OrderBatchRequest>,
    ) -> Result<Response<OrderBatchResponse>, Status> {
        let req = request.into_inner();
        let mode = req.mode;
        let mut results = Vec::new();
        for order in req.orders {
            let result = match self.place_order(Request::new(order.clone())).await {
                Ok(resp) => OrderBatchItemResult {
                    order_id: order.id,
                    instrument_id: order.instrument_id,
                    result: Some(proto::order_batch_item_result::Result::Order(resp.into_inner())),
                },
                Err(err) => {
                    if mode == BatchMode::AllOrNone as i32 {
                        return Err(err);
                    }
                    OrderBatchItemResult {
                        order_id: order.id,
                        instrument_id: order.instrument_id,
                        result: Some(proto::order_batch_item_result::Result::Error(status_from_error(&err))),
                    }
                }
            };
            results.push(result);
        }
        Ok(Response::new(OrderBatchResponse { results }))
    }
    

    async fn cancel_order(
	&self,
	request: Request<CancelOrderRequest>,
    ) -> Result<Response<OrderResponse>, Status> {
	let req = request.into_inner();
	let lane = self.lane_sender_for_instrument(req.instrument_id).await;
        let (tx, rx) = oneshot::channel();
        lane.send(WorkerCommand::Cancel {
            order_id: req.order_id,
            instrument_id: req.instrument_id,
            idempotency_key: req.idempotency_key,
            response: tx,
        })
        .await
        .map_err(|_| Status::internal("Lane worker unavailable"))?;
        let cancelled_order = rx
            .await
            .map_err(|_| Status::internal("Lane worker response dropped"))??;
	Ok(Response::new(order_to_response(cancelled_order)))
    }

    async fn cancel_orders(
        &self,
        request: Request<CancelOrderBatchRequest>,
    ) -> Result<Response<CancelOrderBatchResponse>, Status> {
        let mut results = Vec::new();
        for req in request.into_inner().requests {
            let result = match self.cancel_order(Request::new(req.clone())).await {
                Ok(resp) => proto::cancel_order_batch_item_result::Result::Order(resp.into_inner()),
                Err(err) => proto::cancel_order_batch_item_result::Result::Error(status_from_error(&err)),
            };
            results.push(proto::CancelOrderBatchItemResult {
                order_id: req.order_id,
                instrument_id: req.instrument_id,
                result: Some(result),
            });
        }
        Ok(Response::new(CancelOrderBatchResponse { results }))
    }

    async fn get_order_book(
        &self,
        request: Request<GetOrderBookRequest>,
    ) -> Result<Response<proto::OrderBookResponse>, Status> {
        let req = request.into_inner();
        let depth = req.depth as usize;
        let lane = self.lane_sender_for_instrument(req.instrument_id).await;
        let (tx, rx) = oneshot::channel();
        lane.send(WorkerCommand::Snapshot {
            depth,
            instrument_id: req.instrument_id,
            response: tx,
        })
        .await
        .map_err(|_| Status::internal("Lane worker unavailable"))?;
        let snapshot = rx
            .await
            .map_err(|_| Status::internal("Lane worker response dropped"))??;
        Ok(Response::new(snapshot))
    }

    async fn stream_order_book(
        &self,
        request: Request<StreamOrderBookRequest>,
    ) -> Result<Response<Self::StreamOrderBookStream>, Status> {
        let req = request.into_inner();
        let interval = Duration::from_millis(req.interval_ms.max(100) as u64);
        let service = self.clone();
        let stream = futures::stream::unfold((), move |_| {
            let service = service.clone();
            let req = req.clone();
            async move {
                tokio::time::sleep(interval).await;
                let resp = service
                    .get_order_book(Request::new(GetOrderBookRequest {
                        depth: req.depth,
                        instrument_id: req.instrument_id,
                    }))
                    .await
                    .map(|r| r.into_inner());
                Some((resp, ()))
            }
        });
        Ok(Response::new(Box::pin(stream)))
    }

    async fn get_order_status(
        &self,
        request: Request<GetOrderStatusRequest>,
    ) -> Result<Response<OrderResponse>, Status> {
        let req = request.into_inner();
        let lane = self.lane_sender_for_instrument(req.instrument_id).await;
        let (tx, rx) = oneshot::channel();
        lane.send(WorkerCommand::Status {
            order_id: req.order_id,
            instrument_id: req.instrument_id,
            response: tx,
        })
        .await
        .map_err(|_| Status::internal("Lane worker unavailable"))?;
        let order = rx
            .await
            .map_err(|_| Status::internal("Lane worker response dropped"))??;
        Ok(Response::new(order_to_response(order)))
    }


    async fn get_trade_history(&self, request: Request<GetTradeHistoryRequest>) -> Result<Response<TradeHistoryResponse>, Status> {
	let req = request.into_inner();
	let limit = req.limit as usize;
        let lane = self.lane_sender_for_instrument(req.instrument_id).await;
        let (tx, rx) = oneshot::channel();
        lane.send(WorkerCommand::Trades {
            limit,
            instrument_id: req.instrument_id,
            response: tx,
        })
        .await
        .map_err(|_| Status::internal("Lane worker unavailable"))?;
        let trades = rx
            .await
            .map_err(|_| Status::internal("Lane worker response dropped"))??;
	Ok(Response::new(TradeHistoryResponse { trades }))
    }

    async fn stream_trade_history(
        &self,
        request: Request<StreamTradeHistoryRequest>,
    ) -> Result<Response<Self::StreamTradeHistoryStream>, Status> {
        let req = request.into_inner();
        let interval = Duration::from_millis(req.interval_ms.max(100) as u64);
        let service = self.clone();
        let stream = futures::stream::unfold(0usize, move |mut idx| {
            let service = service.clone();
            let req = req.clone();
            async move {
                tokio::time::sleep(interval).await;
                let resp = service
                    .get_trade_history(Request::new(GetTradeHistoryRequest {
                        limit: req.limit,
                        instrument_id: req.instrument_id,
                    }))
                    .await
                    .map(|r| r.into_inner().trades)
                    .unwrap_or_default();
                if resp.is_empty() {
                    return Some((Err(Status::not_found("No trades yet")), idx));
                }
                idx = (idx + 1) % resp.len();
                Some((Ok(resp[idx].clone()), idx))
            }
        });
        Ok(Response::new(Box::pin(stream)))
    }
}
