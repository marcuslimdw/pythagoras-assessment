use crate::config::Config;
use crate::consumer::{Consumer, MongoDB, Redis};
use crate::producer::OkexProducer;

mod consumer;
mod producer;
mod config;


const INSTRUMENT_IDS: [&str; 1] = ["BTC-USD-SWAP"];

fn main() {
    env_logger::init();
    let Config { websocket_url, mongodb_url, redis_url } = Config::from_env();

    let mut producer = OkexProducer::connect(&websocket_url);
    for instrument_id in Vec::from(INSTRUMENT_IDS) {
        producer.request_orderbook(instrument_id);
    }

    let mut consumers: Vec<Box<dyn Consumer>> = vec![
        Box::new(MongoDB::connect(&mongodb_url)),
        Box::new(Redis::connect(&redis_url))
    ];
    producer.listen(&mut consumers);
}
