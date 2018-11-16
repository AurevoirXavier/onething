mod export;
pub mod proxy;
pub mod redeem;
mod sign_in;

// --- std ---
use std::sync::{Arc, Mutex};

// --- custom ---
use self::proxy::Proxies;

#[derive(Clone)]
pub struct Account<'a> {
    name: String,
    pass: String,
    pub session: reqwest::Client,
    cookie: reqwest::header::HeaderMap,
    proxies: Option<&'a Arc<Mutex<Proxies>>>,
}

impl<'a> Account<'a> {
    pub fn new(name: &str, pass: &str, proxy: Option<&'a Arc<Mutex<Proxies>>>) -> Account<'a> {
        Account {
            name: name.to_owned(),
            pass: pass.to_owned(),
            session: reqwest::Client::new(),
            cookie: reqwest::header::HeaderMap::new(),
            proxies: proxy,
        }
    }
}
