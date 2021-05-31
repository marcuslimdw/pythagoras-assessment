use serde::{Deserialize, Serialize};


pub enum OrderbookSide {
    Ask,
    Bid
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct OrderbookArg {
    pub channel: String,
    #[serde(rename = "instId")]
    pub inst_id: String,
}

pub type Order = (String, String, String, String);

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct OrderbookDataElement {
    pub asks: Vec<Order>,
    pub bids: Vec<Order>,
    pub ts: String,
    pub checksum: i64,
}

pub type OrderbookData = Vec<OrderbookDataElement>;

#[derive(Deserialize, Serialize, PartialEq, Debug)]
#[serde(untagged)]
pub enum WebsocketPushData {
    Orderbook {
        arg: OrderbookArg,
        action: String,
        data: OrderbookData,
    }
}

