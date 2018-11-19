// --- std ---
use std::{
    u128,
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};

// --- external ---
use cpython::{NoArgs, ObjectProtocol, Python, PyDict};
use emerald_core::{
    ToHex,
    keystore::KeyFile,
};
use reqwest::Proxy;
use serde_json::{Value, from_str};

// --- custom ---
use crate::util::{
    default_client_builder,
    init::{CONF, GET_BALANCE_API, GET_TRANSACTION_COUNT_API, SEND_RAW_TRANSACTION_API, TRANSACTIONS, TRANSACTION_HEADERS, WALLETS},
};
use super::get_info;

#[derive(Clone)]
pub struct Transaction<'a> {
    to: &'a str,
    value: &'a str,
    gas_limit: &'a str,
    data: &'a str,
    from: String,
    signed_raw_transaction: String,
}

impl<'a> Transaction<'a> {
    pub fn new(to: &'a str, value: &'a str, gas_limit: &'a str, data: &'a str) -> Transaction<'a> {
        Transaction {
            to,
            value,
            gas_limit,
            data,
            from: String::new(),
            signed_raw_transaction: String::new(),
        }
    }

    pub fn sign(&mut self, wallet: &PathBuf) -> &mut Self {
        let nonce = {
            let from = wallet.file_name().unwrap().to_str().unwrap();
            self.from = from.to_owned();

            get_info(GET_TRANSACTION_COUNT_API, from)
        };

        let gil = Python::acquire_gil();
        let py = gil.python();
        let web3 = py.import("web3").unwrap();
        let signed_transaction = {
            let transaction = {
                let transaction = PyDict::new(py);
                transaction.set_item(py, "gas", self.gas_limit).unwrap();
                transaction.set_item(py, "gasPrice", "0x174876e800").unwrap();
                transaction.set_item(py, "nonce", nonce).unwrap();
                transaction.set_item(py, "data", self.data).unwrap();

                let web3 = web3.get(py, "Web3").unwrap();
                let to = web3.call_method(py, "toChecksumAddress", (self.to, ), None).unwrap();

                transaction.set_item(py, "to", to).unwrap();
                transaction.set_item(py, "value", format!("{:#x}", self.value.parse::<u128>().unwrap())).unwrap();

                transaction
            };

//            println!("{:?}", transaction.items(py));  // TODO Debug

            let private_key = {
                let mut key_file = String::new();

                // TODO File not found
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

            web3.get(py, "Account")
                .unwrap()
                .call_method(py, "signTransaction", (transaction, private_key), None)
                .unwrap()
        };

//    println!("{:?}", signed_transaction);  // TODO Debug

        self.signed_raw_transaction = signed_transaction.getattr(py, "rawTransaction")
            .unwrap()
            .call_method(py, "hex", NoArgs, None)
            .unwrap()
            .extract(py)
            .unwrap();

        self
    }

    pub fn send(&mut self) {
        loop {
            match default_client_builder(10)
                .default_headers(TRANSACTION_HEADERS.clone())
                .proxy(Proxy::https(&CONF.transaction_proxy).unwrap())
                .build()
                .unwrap()
                .post(SEND_RAW_TRANSACTION_API)
                .json(&json!({
                    "jsonrpc": "2.0",
                    "method": "eth_sendRawTransaction",
                    "params": [self.signed_raw_transaction],
                    "id": 1
                })).send() {
                Ok(mut resp) => {
                    let data = resp.text().unwrap();
//                println!("{}", data);  // TODO Debug
                    if data.contains('<') { continue; }

                    let data: Value = from_str(&data).unwrap();
                    if let Some(hash) = data.get("result") {
                        let hash = hash.as_str().unwrap();
                        println!("Succeed, transaction hash: [{}]", hash);
                        writeln!(TRANSACTIONS.lock().unwrap(), "{}", hash).unwrap();
                        break;
                    } else {
                        println!("{}", data);
                        self.sign(&WALLETS.lock().unwrap().next()).send();
                    }
                }
                Err(_e) => {
//                    println!("{:?}", _e);  // TODO Debug
                    continue;
                }
            }
        }
    }
}

pub fn check_balance(from: &str, value: &str, gas_limit: &str) -> bool {
    let balance = u128::from_str_radix(&get_info(GET_BALANCE_API, from)[2..], 16).unwrap();
    let gas_limit = u128::from_str_radix(&gas_limit[2..], 16).unwrap();
    let gas_price = 0x174876e800;
    let value = value.parse::<u128>().unwrap();

    balance >= value + gas_limit * gas_price
}
