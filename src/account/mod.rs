mod export;
pub mod redeem;
mod sign_in;

// --- std ---
use std::sync::{Arc, Mutex};

// --- custom ---
use crate::util::proxy::Proxies;

#[derive(Clone)]
pub struct Account<'a> {
    username: String,
    password: String,
    pub session: reqwest::Client,
    cookie: reqwest::header::HeaderMap,
    pub proxies: Option<&'a Arc<Mutex<Proxies>>>,
}

impl<'a> Account<'a> {
    pub fn new(username: &str, password: &str, proxies: Option<&'a Arc<Mutex<Proxies>>>) -> Account<'a> {
        Account {
            username: username.to_owned(),
            password: password.to_owned(),
            session: reqwest::Client::new(),
            cookie: reqwest::header::HeaderMap::new(),
            proxies,
        }
    }

    pub fn from_str(info: &str) -> Account<'a> {
        let account: Vec<&str> = info.split('=').collect();
        Account {
            username: account[0].to_owned(),
            password: account[1].to_owned(),
            session: reqwest::Client::new(),
            cookie: reqwest::header::HeaderMap::new(),
            proxies: None,
        }
    }

    pub fn with_proxies(mut self, proxies: &'a Arc<Mutex<Proxies>>) -> Self {
        self.proxies = Some(proxies);
        self
    }
}
