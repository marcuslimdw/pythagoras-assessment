use log::{warn};
use redis::{Client, Commands, Connection};

use crate::producer::{Order, OrderbookDataElement, OrderbookSide, WebsocketPushData};
use crate::consumer::consumer::Consumer;

pub struct Redis {
    connection: Connection
}

impl Consumer for Redis {
    fn connect(url: &str) -> Self {
        let client = Client::open(url).expect(&format!("Invalid Redis URL {}.", url));
        let connection = client.get_connection().expect(&format!("Couldn't connect to Redis at {}", url));
        Redis { connection }
    }

    fn write(&mut self, data: &WebsocketPushData) {
        use crate::producer::WebsocketPushData::*;

        match data {
            Orderbook { arg, data, .. } => self.write_orderbook(&arg.inst_id, &data[0])
        };
    }
}

impl Redis {
    fn write_orderbook(&mut self, instrument: &str, data: &OrderbookDataElement) {
        let OrderbookDataElement { asks, bids, .. } = data;
        self.write_side(instrument, OrderbookSide::Ask, asks);
        self.write_side(instrument, OrderbookSide::Bid, bids);
    }

    fn write_side(&mut self, instrument: &str, side: OrderbookSide, orders: &Vec<Order>) {
        let side_name = match side {
            OrderbookSide::Ask => "asks",
            OrderbookSide::Bid => "bids"
        };
        let key = format!("{}:{}:{}", "orderbook", instrument, side_name);
        for (price, size, liquidated_orders, total_orders) in orders.iter() {
            let value = format!("{},{},{}", size, liquidated_orders, total_orders);
            match self.connection.hset::<&str, &str, &str, ()>(&key, price, &value) {
                Ok(_) => (),
                Err(_) => warn!("Failed to write to Redis orderbook.")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn test_write_side_ask() {
        // TODO: Mock connection.
        let mut redis = Redis::connect("");
        let channel = "TEST-CHANNEL";
        let side = OrderbookSide::Ask;
        let orders = vec![
            (String::from("33919.8"), String::from("105"), String::from("0"), String::from("2")),
            (String::from("33924.1"), String::from("32"), String::from("0"), String::from("1"))
        ];
        redis.write_side(channel, side, &orders);
        // Assert that hset method of mock object is called.
    }
}
