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
}
