pub mod init;
pub mod proxy;

// --- std ---
use std::{
    u128,
    fs::{File, OpenOptions},
    io::{Read, Write},
    time::Duration,
};

// --- external ---
use regex::Regex;
use reqwest::{Client, ClientBuilder};

// --- custom ---
use self::init::CONF;

pub fn default_client_builder(timeout: u64) -> ClientBuilder {
    let timeout = if timeout == 0 { CONF.request_timeout } else { timeout };
    Client::builder()
        .danger_accept_invalid_certs(true)
        .danger_accept_invalid_hostnames(true)
        .gzip(true)
        .timeout(Duration::from_secs(timeout))
}

pub fn format_hex(hex: &str) -> f64 { u128::from_str_radix(&hex[2..], 16).unwrap() as f64 / 10f64.powi(18) }

pub fn format_code() {
    let codes = {
        let mut codes = String::new();
        let mut f = File::open(&format!("codes_{}.txt", CONF.date)).unwrap();
        f.read_to_string(&mut codes).unwrap();

        codes
    };

    for line in codes.lines() {
        
    }
}
