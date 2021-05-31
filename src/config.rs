use std::env;

use log::warn;

const WEBSOCKET_URL_KEY: &str = "PYTHAGORAS_WEBSOCKET_URL";
const MONGODB_URL_KEY: &str = "PYTHAGORAS_MONGODB_URL";
const REDIS_URL_KEY: &str = "PYTHAGORAS_REDIS_URL";

const DEFAULT_WEBSOCKET_URL: &str = "wss://wspap.okex.com:8443/ws/v5/public?brokerId=9999";
const DEFAULT_MONGODB_URL: &str = "mongodb://localhost:27017/";
const DEFAULT_REDIS_URL: &str = "redis://localhost:6379/";

pub struct Config {
    pub websocket_url: String,
    pub mongodb_url: String,
    pub redis_url: String
}

// This is a dependency injection example. To properly unit test the consumers and producer, this
// approach can also be taken. Mocking is a lot harder in Rust than in many other languages,
// due to its strong type system and relative immaturity, but the basic idea still remains.

trait QueryEnvironment {
    fn get_or_default(&self, key: &str, default: &str) -> String;
}

struct Environment;

impl QueryEnvironment for Environment {
    fn get_or_default(&self, key: &str, default: &str) -> String {
        match env::var(key) {
            Ok(value) => value,
            Err(e) => {
                warn!("Couldn't load value for key {} from environment due to error {:?}. Using default {}.", key, e, default);
                String::from(default)
            }
        }
    }
}

impl Config {
    fn new(environment: Box<dyn QueryEnvironment>) -> Self {
        let websocket_url = environment.get_or_default(WEBSOCKET_URL_KEY, DEFAULT_WEBSOCKET_URL);
        let mongodb_url = environment.get_or_default(MONGODB_URL_KEY,DEFAULT_MONGODB_URL);
        let redis_url = environment.get_or_default(REDIS_URL_KEY, DEFAULT_REDIS_URL);
        Config { websocket_url, mongodb_url, redis_url }
    }

    pub fn from_env() -> Self {
        let environment = Box::new(Environment);
        Config::new(environment)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockEnvironment;

    impl QueryEnvironment for MockEnvironment {
        fn get_or_default(&self, key: &str, default: &str) -> String {
            if key == MONGODB_URL_KEY { String::from(default) } else { format!("from_{}", key) }
        }
    }

    #[test]
    fn test_new() {
        let environment = Box::new(MockEnvironment);
        let Config { websocket_url, mongodb_url, redis_url } = Config::new(environment);
        assert_eq!(websocket_url, "from_PYTHAGORAS_WEBSOCKET_URL");
        assert_eq!(mongodb_url, DEFAULT_MONGODB_URL);
        assert_eq!(redis_url, "from_PYTHAGORAS_REDIS_URL");
    }
}
