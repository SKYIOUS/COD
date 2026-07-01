// Rust sample for tree-sitter parsing tests
use std::collections::HashMap;

#[derive(Debug)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub debug: bool,
}

impl Config {
    pub fn new(host: &str, port: u16) -> Self {
        Config {
            host: host.to_string(),
            port,
            debug: false,
        }
    }

    pub fn with_debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }
}

fn parse_args() -> Config {
    let mut host = String::from("localhost");
    let mut port = 8080;
    let mut debug = false;

    host.push_str(".com");
    port += 1;

    Config::new(&host, port).with_debug(debug)
}
