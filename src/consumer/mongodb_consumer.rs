use log::warn;
use mongodb::sync::Client;

use crate::consumer::consumer::Consumer;
use crate::producer::{OrderbookDataElement, WebsocketPushData};
use crate::producer::WebsocketPushData::Orderbook;

pub struct MongoDB {
    client: Client,
}

impl Consumer for MongoDB {
    fn connect(url: &str) -> Self {
        let client = Client::with_uri_str(url).expect(&format!("Invalid MongoDB URL {}", url));
        MongoDB { client }
    }

    fn write(&mut self, data: &WebsocketPushData) {
        match data {
            Orderbook { arg, data, .. } => self.write_orderbook(&arg.inst_id, &data[0])
        }
    }
}

impl MongoDB {
    fn write_orderbook(&mut self, instrument: &str, data: &OrderbookDataElement) {
        let database = self.client.database("orderbook");
        let collection = database.collection::<OrderbookDataElement>(instrument);
        match collection.insert_one(data, None) {
            Ok(_) => (),
            Err(_) => warn!("Failed to write to MongoDB orderbook.")
        };
    }
}
