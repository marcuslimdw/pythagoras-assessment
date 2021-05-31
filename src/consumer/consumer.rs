use crate::producer::WebsocketPushData;

pub trait Consumer {
    fn connect(url: &str) -> Self where Self: Sized;

    fn write(&mut self, data: &WebsocketPushData);
}
