pub mod init;
pub mod proxy;

// --- std ---
use std::{
    u128,
    time::Duration,
};

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

pub fn format_hex(hex: &str) -> f64 { u128::from_str_radix(&hex[2..], 16).unwrap() as f64 / 10f64.powi(18) }
