use std::sync::{Arc, Mutex};
use tonic::{transport::Server, Request, Response, Status};
use rust_decimal::Decimal;
use std::str::FromStr;
use prost_types::Timestamp;

use crate::core::MatchingEngine; // Updated import
use crate::proto;
use crate::proto::order_book_service_server::{OrderBookService as GrpcService, OrderBookServiceServer};
use crate::proto::{OrderRequest, OrderResponse, GetOrderBookRequest, OrderBookResponse, GetOrderStatusRequest, CancelOrderRequest};
use crate::core::{Order, OrderType, Side};
use crate::proto::{GetTradeHistoryRequest, TradeHistoryResponse, Trade as ProtoTrade};

pub struct OrderBookService {
    engine: Arc<Mutex<MatchingEngine>>,
}

impl OrderBookService {
    pub fn new(engine: MatchingEngine) -> Self {
        Self {
            engine: Arc::new(Mutex::new(engine))
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
}

#[tonic::async_trait]
impl GrpcService for OrderBookService {
    async fn place_order(
        &self,
        request: Request<OrderRequest>,
    ) -> Result<Response<OrderResponse>, Status> {
        let req = request.into_inner();
        let price = Decimal::from_str(&req.price)
            .map_err(|_| Status::invalid_argument("Invalid price format"))?;
        let quantity = Decimal::from_str(&req.quantity)
            .map_err(|_| Status::invalid_argument("Invalid quantity format"))?;

        let order = Order::new(
            req.id,
            price,
            quantity,
            if req.side == 0 { Side::Bid } else { Side::Ask },
            if req.order_type == 0 { OrderType::Limit } else { OrderType::Market },
        );

        let result = self.engine
            .lock()
            .map_err(|_| Status::internal("Lock error"))?
            .place_order(order);

        Ok(Response::new(OrderResponse {
            id: result.id,
            price: result.price.to_string(),
            quantity: result.quantity.to_string(),
            remaining_quantity: result.remaining_quantity.to_string(),
            side: result.side as i32,
            order_type: result.order_type as i32,
            status: result.status as i32,
            timestamp: Some(Timestamp {
                seconds: result.timestamp.timestamp(),
                nanos: result.timestamp.timestamp_subsec_nanos() as i32,
            }),
        }))
    }

    async fn cancel_order(
	&self,
	request: Request<CancelOrderRequest>,
    ) -> Result<Response<OrderResponse>, Status> {
	let order_id = request.into_inner().order_id;
	let cancelled_order = self.engine
            .lock()
            .map_err(|_| Status::internal("Lock error"))?
            .cancel_order(order_id)
            .ok_or_else(|| Status::not_found("Order not found or cannot be cancelled"))?;
	Ok(Response::new(OrderResponse {
            id: cancelled_order.id,
            price: cancelled_order.price.to_string(),
            quantity: cancelled_order.quantity.to_string(),
            remaining_quantity: cancelled_order.remaining_quantity.to_string(),
            side: cancelled_order.side as i32,
            order_type: cancelled_order.order_type as i32,
            status: cancelled_order.status as i32,
            timestamp: Some(Timestamp {
                seconds: cancelled_order.timestamp.timestamp(),
                nanos: cancelled_order.timestamp.timestamp_subsec_nanos() as i32,
            }),
        }))
    }

    async fn get_order_book(
        &self,
        request: Request<GetOrderBookRequest>,
    ) -> Result<Response<OrderBookResponse>, Status> {
        let depth = request.into_inner().depth as usize;
        let (bids, asks) = self.engine
            .lock()
            .map_err(|_| Status::internal("Lock error"))?
            .get_order_book(depth);

        Ok(Response::new(OrderBookResponse {
            bids: bids.into_iter()
                .map(|(price, qty)| proto::OrderBookLevel {
                    price: price.to_string(),
                    quantity: qty.to_string(),
                })
                .collect(),
            asks: asks.into_iter()
                .map(|(price, qty)| proto::OrderBookLevel {
                    price: price.to_string(),
                    quantity: qty.to_string(),
                })
                .collect(),
        }))
    }

    async fn get_order_status(
        &self,
        request: Request<GetOrderStatusRequest>,
    ) -> Result<Response<OrderResponse>, Status> {
        let order_id = request.into_inner().order_id;

        let order = self.engine
            .lock()
            .map_err(|_| Status::internal("Lock error"))?
            .get_order_status(order_id)
            .ok_or_else(|| Status::not_found("Order not found"))?
            .clone();

        Ok(Response::new(OrderResponse {
            id: order.id,
            price: order.price.to_string(),
            quantity: order.quantity.to_string(),
            remaining_quantity: order.remaining_quantity.to_string(),
            side: order.side as i32,
            order_type: order.order_type as i32,
            status: order.status as i32,
            timestamp: Some(prost_types::Timestamp {
                seconds: order.timestamp.timestamp(),
                nanos: order.timestamp.timestamp_subsec_nanos() as i32,
            }),
        }))
    }


    async fn get_trade_history(&self, request: Request<GetTradeHistoryRequest>) -> Result<Response<TradeHistoryResponse>, Status> {
	let limit = request.into_inner().limit as usize;

	let trades = self.engine
	    .lock()
	    .map_err(|_| Status::internal("lock error"))?
	    .get_trade_history(Some(limit))
	    .into_iter()
	    .map(|trade| ProtoTrade{
		maker_order_id: trade.maker_order_id,
                taker_order_id: trade.taker_order_id,
                price: trade.price.to_string(),
                quantity: trade.quantity.to_string(),
                side: trade.side as i32,
                timestamp: Some(Timestamp {
                    seconds: trade.timestamp.timestamp(),
                    nanos: trade.timestamp.timestamp_subsec_nanos() as i32,
                }),
	    })
	    .collect();

	Ok(Response::new(TradeHistoryResponse { trades }))
    }
}
