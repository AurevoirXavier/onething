// --- std ---
use std::{
    u128,
    collections::HashSet,
    fs::{File, create_dir, read_dir},
    io::{Read, Write, stdin, stdout},
    iter::Cycle,
    path::{Path, PathBuf},
    vec::IntoIter,
};

// --- external ---
use cpython::{NoArgs, ObjectProtocol, Python, PyDict};
use emerald_core::{
    ToHex,
    keystore::{KdfDepthLevel, KeyFile},
};
use reqwest::Proxy;
use serde_json::{Value, from_str};

// --- custom ---
use crate::util::{
    default_client_builder,
    init::{CONF, GET_BALANCE_API, GET_TRANSACTION_COUNT_API, SEND_RAW_TRANSACTION_API, TRANSACTION_HEADERS, WALLETS},
};

pub struct Wallets {
    all: HashSet<PathBuf>,
    available: Cycle<IntoIter<PathBuf>>,
}

impl Wallets {
    pub fn new() -> Wallets {
        let all: Vec<PathBuf> = read_dir("wallets")
            .unwrap()
            .map(|d| d.unwrap().path())
            .filter(|path| path.file_name().unwrap().to_str().unwrap().starts_with("0x"))
            .collect();

        Wallets {
            all: all.iter().cloned().collect(),
            available: all.into_iter().cycle(),
        }
    }

    pub fn update(&mut self, unavailable: &PathBuf) {
        if self.all.len() == 1 { panic!("All wallets are unavailable."); }

        self.all.remove(unavailable);
        self.available = self.all
            .iter()
            .cloned()
            .collect::<Vec<PathBuf>>()
            .into_iter()
            .cycle();
    }

    pub fn next(&mut self) -> PathBuf { self.available.next().unwrap() }
}

fn get_info(url: &str, address: &str) -> String {
    loop {
        if let Ok(mut resp) = default_client_builder()
            .build()
            .unwrap()
            .post(url)
            .json(&json!({
                "jsonrpc": "2.0",
                "method": "eth_getTransactionCount",  // TODO Might be changed in the future
                "params": [address, "latest"],
                "id": 1
             })).send() {
            let data = resp.text().unwrap();
//            println!("{}", data);  // TODO Debug
            if data.contains('<') { continue; }

            let order: Value = from_str(&data).unwrap();
            if let Some(result) = order.get("result") {
                return result.as_str().unwrap().to_owned();
            } else { continue; }
        } else { continue; }
    }
}

pub fn gen_wallet() {
    let mut amount = String::new();
    print!("Amount: ");
    stdout().flush().unwrap();
    stdin().read_line(&mut amount).unwrap();

    let mut password = String::new();
    print!("Password (Enter for default password `123456789`): ");
    stdout().flush().unwrap();
    stdin().read_line(&mut password).unwrap();
    if password.trim().is_empty() { password = "123456789".to_string(); }

    {
        let dir = Path::new("new-wallets");
        if !dir.exists() { create_dir(dir).unwrap(); }
    }
    for i in 1..=amount.trim().parse::<u64>().unwrap() {
        let key_file = KeyFile::new(
            &password,
            &KdfDepthLevel::Normal,
            None,
            None,
        ).unwrap();
        key_file.flush("new-wallets", Some(&key_file.address.to_string())).unwrap();

        println!("No.{} wallet was generated.", i);
    }
}

pub fn sign_transaction(wallet: PathBuf, to: &str, value: &str, gas_limit: &str, data: &str) -> String {
    let gas_limit = if gas_limit.is_empty() { "0x186a0" } else { gas_limit };
    let nonce = get_info(GET_TRANSACTION_COUNT_API, wallet.file_name().unwrap().to_str().unwrap());
    let value = format!("{:#x}", value.parse::<u128>().unwrap());

    let private_key = {
        let mut key_file = String::new();
        File::open(wallet)
            .unwrap()
            .read_to_string(&mut key_file)
            .unwrap();

        KeyFile::decode(key_file)
            .unwrap()
            .decrypt_key("123456789")
            .unwrap()
            .to_hex()
    };

    let gil = Python::acquire_gil();
    let py = gil.python();
    let web3 = py.import("web3").unwrap();

    let transaction = {
        let transaction = PyDict::new(py);
        transaction.set_item(py, "gas", gas_limit).unwrap();
        transaction.set_item(py, "gasPrice", "0x174876e800").unwrap();
        transaction.set_item(py, "nonce", nonce).unwrap();
        transaction.set_item(py, "data", data).unwrap();

        let web3 = web3.get(py, "Web3").unwrap();
        let to = web3.call_method(py, "toChecksumAddress", (to, ), None).unwrap();

        transaction.set_item(py, "to", to).unwrap();
        transaction.set_item(py, "value", value).unwrap();

        transaction
    };

//    println!("{:?}", transaction.items(py));  // TODO Debug

    let account = web3.get(py, "Account").unwrap();
    let signed_transaction = account.call_method(py, "signTransaction", (transaction, private_key), None).unwrap();

//    println!("{:?}", signed_transaction);  // TODO Debug

    signed_transaction.getattr(py, "rawTransaction")
        .unwrap()
        .call_method(py, "hex", NoArgs, None)
        .unwrap()
        .extract(py)
        .unwrap()
}

pub fn send_transaction(signed_transaction: &str) {
    loop {
        match default_client_builder()
            .default_headers(TRANSACTION_HEADERS.clone())
            .proxy(Proxy::https(&CONF.transaction_proxy).unwrap())
            .build()
            .unwrap()
            .post(SEND_RAW_TRANSACTION_API)
            .json(&json!({
                "jsonrpc": "2.0",
                "method": "eth_sendRawTransaction",
                "params": [signed_transaction],
                "id": 1
            })).send() {
            Ok(mut resp) => {
//                println!("{}", resp.text().unwrap());  // TODO Debug
                let data = resp.text().unwrap();
                if data.contains('<') { continue; } else if data.contains("result") { break; } else { println!("{}", data); }
            }
            Err(e) => {
                println!("{:?}", e);
                continue;
            }
        }
    }
}

pub fn sign_transaction_with_random_wallet(to: &str, value: &str, gas_limit: &str, data: &str) -> String {
    let mut wallet;
    let mut from;
    loop {
        let mut guard = WALLETS.lock().unwrap();
        wallet = guard.next();

        from = wallet.file_name()
            .unwrap()
            .to_str()
            .unwrap();

        let balance = u128::from_str_radix(&get_info(GET_BALANCE_API, from)[2..], 16).unwrap();
        let gas_limit = u128::from_str_radix(&gas_limit[2..], 16).unwrap();
        let gas_price = 0x174876e800u128;

        if balance > gas_limit * gas_price { break; } else {
            guard.update(&wallet);
            continue;
        }
    }

    sign_transaction(wallet, to, value, gas_limit, data)
}

pub fn settle_accounts() {}

#[test]
fn test() {
    sign_transaction_with_random_wallet(
        "0xdce69e7f233b8876019093e0c8abf75e33dd8603",
        "100000000000000000",
        "",
        "",
    );
}
