// --- std ---
use std::{
    fs::{File, OpenOptions},
    io::prelude::*,
    sync::{Arc, Mutex},
    vec::IntoIter,
};

// --- external ---
use reqwest::header::HeaderMap;
use serde_json::from_reader;

// --- custom ---
use crate::wallet::Wallets;
use super::proxy::Proxies;

pub const GET_BALANCE_API: &'static str = "https://walletapi.onethingpcs.com/getBalance";
pub const GET_TRANSACTION_COUNT_API: &'static str = "https://walletapi.onethingpcs.com/getTransactionCount";
pub const ORDER_INFO_API: &'static str = "https://api-mall.onethingpcs.com/order/info";
pub const ORDER_LIST_API: &'static str = "https://api-mall.onethingpcs.com/order/list";
pub const SEND_RAW_TRANSACTION_API: &'static str = "https://walletapi.onethingpcs.com/sendRawTransaction";
pub const SIGN_IN_API: &'static str = "https://api-accw.onethingpcs.com/user/login";
pub const SUBMIT_ORDER_API: &'static str = "https://api-mall.onethingpcs.com/order/submitorder";

lazy_static! {
    pub static ref ACCOUNTS: Vec<String> = {
        let mut f = File::open("accounts.txt").unwrap();
        let mut accounts = String::new();
        f.read_to_string(&mut accounts).unwrap();

        accounts.lines().map(|line| line.to_owned()).collect()
    };

    pub static ref CODES: Arc<Mutex<File>> = {
        Arc::new(Mutex::new(OpenOptions::new()
            .append(true)
            .create(true)
            .open(format!("codes_{}.txt", CONF.date))
            .unwrap()))
    };

    pub static ref DETECTORS: Arc<Mutex<IntoIter<String>>> = {
        let mut f = File::open("detectors.txt").unwrap();
        let mut detectors = String::new();
        f.read_to_string(&mut detectors).unwrap();

        Arc::new(Mutex::new(
            detectors
                .lines()
                .map(|line| line.to_owned())
                .collect::<Vec<String>>()
                .into_iter()
        ))
    };

    pub static ref ORDERS: Arc<Mutex<File>> = {
        Arc::new(Mutex::new(OpenOptions::new()
            .append(true)
            .create(true)
            .open(format!("orders_{}.txt", CONF.date))
            .unwrap()))
    };

    pub static ref PROXIES: Arc<Mutex<Proxies>> = Arc::new(Mutex::new(Proxies::new()));

    pub static ref TRANSACTIONS: Arc<Mutex<File>> = {
        Arc::new(Mutex::new(OpenOptions::new()
            .append(true)
            .create(true)
            .open(format!("transactions_{}.txt", CONF.date))
            .unwrap()))
    };

    pub static ref TRANSACTION_HEADERS: HeaderMap = {
        let mut headers = HeaderMap::new();
        headers.insert("Nc", "IN".parse().unwrap());

        headers
    };

    pub static ref WALLETS: Arc<Mutex<Wallets>> = Arc::new(Mutex::new(Wallets::new()));
}

#[derive(Deserialize)]
pub struct Conf {
    pub proxy_pool_api: String,
    pub transaction_proxy: String,
    pub date: String,
    pub request_timeout: u64,
    pub account_per_thread: usize,
    pub wallet_per_thread: usize,
    pub export_with_proxy: bool,
    pub kinds: Vec<u8>,
}

fn load_conf() -> Conf { from_reader(File::open("conf.json").unwrap()).unwrap() }

lazy_static! { pub static ref CONF: Conf = load_conf(); }
