pub mod init;
pub mod proxy;

// --- std ---
use std::time::Duration;

// --- external ---
use reqwest::{Client, ClientBuilder};

// --- custom ---
use self::init::CONF;

pub fn default_client_builder() -> ClientBuilder {
    Client::builder()
        .danger_accept_invalid_certs(true)
        .danger_accept_invalid_hostnames(true)
        .gzip(true)
        .timeout(Duration::from_secs(CONF.request_timeout))
}
