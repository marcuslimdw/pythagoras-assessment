use log::{error, info, warn};
use tungstenite::{connect, Message, WebSocket};
use tungstenite::client::AutoStream;
use tungstenite::error::Error as TungsteniteError;
use url::Url;

use crate::consumer::Consumer;
use crate::producer::WebsocketPushData;

#[derive(Debug)]
enum ProducerError {
    ReadError(TungsteniteError),
    NonTextError(TungsteniteError),
    DeserializationError(String),
}

pub struct OkexProducer {
    socket: WebSocket<AutoStream>,
}

impl OkexProducer {
    pub fn connect(endpoint: &str) -> Self {
        let url = Url::parse(endpoint)
            .expect(&format!("Invalid websocket endpoint {}.", endpoint));
        let (socket, _) = connect(url)
            .expect(&format!("Can't connect to websocket endpoint {}.", endpoint));
        info!("Successfully connected to websocket endpoint.");
        OkexProducer { socket }
    }

    pub fn request_orderbook(&mut self, instrument_id: &str) {
        let message = OkexProducer::format_orderbook_request(instrument_id);
        match self.socket.write_message(message).and_then(|_| self.socket.read_message()) {
            Ok(_) => info!("Successfully requested orderbook {}.", instrument_id),
            Err(_) => warn!("Couldn't request orderbook {}.", instrument_id)
        };
    }

    pub fn listen(&mut self, consumers: &mut Vec<Box<dyn Consumer>>) {
        use ProducerError::*;
        loop {
            match self.socket
                .read_message()
                .map_err(|e| ReadError(e))
                .and_then(|m| OkexProducer::parse_message(m)) {
                Ok(data) => for consumer in &mut *consumers { consumer.write(&data) },
                Err(error) => OkexProducer::handle_error(error)
            };
        }
    }

    fn parse_message(message: Message) -> Result<WebsocketPushData, ProducerError> {
        use ProducerError::*;
        message
            .into_text()
            .map_err(|e| NonTextError(e))
            .and_then(|text| serde_json::from_str(&text).map_err(|_| DeserializationError(text)))
    }

    fn handle_error(error: ProducerError) {
        // It's possible to reconnect when encountering errors by making this a method, but that
        // would complicate testing.
        use TungsteniteError::*;
        use ProducerError::*;
        match error {
            ReadError(ConnectionClosed) | ReadError(AlreadyClosed) => panic!("Websocket connection closed."),
            ReadError(Io(_)) => panic!("Websocket IO error."),
            NonTextError(_) => error!("Received non-text data in websocket response."),
            DeserializationError(text) => error!("Couldn't deserialize data into any known schema. JSON received: {}", text),
            other => error!("An unknown error occurred: {:?}.", other)
        };
    }

    fn format_orderbook_request(instrument_id: &str) -> Message {
        let request = format!("{{\
    \"op\": \"subscribe\",
    \"args\": [
        {{
            \"channel\": \"books\",
            \"instId\": \"{}\"
        }}
    ]
}}", instrument_id);
        Message::Text(request.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::ProducerError::*;

    #[test]
    fn test_parse_message_non_text() {
        let data: Vec<u8> = vec![195, 28]; // Invalid UTF-8 code point.
        let message = Message::binary(data);
        let actual = OkexProducer::parse_message(message);
        assert!(matches!(actual, Result::Err(NonTextError(_))));
    }

    #[test]
    fn test_parse_message_invalid_json() {
        let data = String::from("{\"test_data\": 1}");
        let message = Message::text(data);
        let actual = OkexProducer::parse_message(message);
        assert!(matches!(actual, Result::Err(DeserializationError(_))));
    }

    #[test]
    fn test_format_orderbook_request() {
        use tungstenite::Message;

        let expected = Message::Text("{\
    \"op\": \"subscribe\",
    \"args\": [
        {
            \"channel\": \"books\",
            \"instId\": \"TEST-PAIR\"
        }
    ]
}".into());
        let result = OkexProducer::format_orderbook_request("TEST-PAIR");
        assert_eq!(result, expected)
    }
}
