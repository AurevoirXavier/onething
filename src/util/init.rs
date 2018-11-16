// --- std ---
use std::{
    fs::{File, read_dir},
    io::prelude::*,
    iter::Cycle,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    vec::IntoIter,
};

// --- external ---
use serde_json::from_reader;

// --- custom ---
use super::proxy::Proxies;

pub const ORDER_LIST_API: &'static str = "https://api-mall.onethingpcs.com/orders/list";
pub const SIGN_IN_API: &'static str = "https://api-accw.onethingpcs.com/user/login";
pub const SUBMIT_ORDER_API: &'static str = "https://api-mall.onethingpcs.com/orders/submitorder";

lazy_static! {
    pub static ref ACCOUNTS: Vec<String> = {
        let mut f = File::open(Path::new("accounts.txt")).unwrap();
        let mut accounts = String::new();
        f.read_to_string(&mut accounts).unwrap();

        accounts.lines().map(|line| line.to_owned()).collect()
    };

    pub static ref DETECTORS: Arc<Mutex<IntoIter<String>>> = {
        let mut f = File::open(Path::new("detectors.txt")).unwrap();
        let mut detectors = String::new();
        f.read_to_string(&mut detectors).unwrap();

        Arc::new(Mutex::new(detectors.lines().map(|line| line.to_owned()).collect::<Vec<String>>().into_iter()))
    };

    pub static ref ORDERS: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));

    pub static ref PROXIES: Arc<Mutex<Proxies>> = Arc::new(Mutex::new(Proxies::new()));

    pub static ref WALLETS: Arc<Mutex<Cycle<IntoIter<PathBuf>>>> = Arc::new(Mutex::new(
        read_dir("wallets")
            .unwrap()
            .map(|d| d.unwrap().path())
            .collect::<Vec<PathBuf>>()
            .into_iter()
            .cycle()
    ));
}

#[derive(Deserialize)]
pub struct Conf {
    pub account_per_thread: usize,
    pub kinds: Vec<u8>,
    pub proxy_pool_api: String,
    pub request_timeout: u64,
}

fn load_conf() -> Conf {
    from_reader(
        File::open(Path::new("conf.json")
        ).unwrap()
    ).unwrap()
}

lazy_static! { pub static ref CONF: Conf = load_conf(); }