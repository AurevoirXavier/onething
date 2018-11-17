// --- std ---
use std::{
    fs::{File, create_dir, read_dir},
    io::{Read, Write, stdin, stdout},
    path::Path,
};

// --- external ---
use cpython::{ObjectProtocol, Python, PyDict};
use emerald_core::{
    ToHex,
    keystore::{KdfDepthLevel, KeyFile}
};
use reqwest::Proxy;
use serde_json::{Value, from_str};

// --- custom ---
use crate::util::{
    default_client_builder,
    init::{CONF, GET_BALANCE_API, GET_TRANSACTION_COUNT_API, SEND_RAW_TRANSACTION_API, TRANSACTION_HEADERS, WALLETS},
};

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
        key_file.flush(Path::new("new-wallets"), Some(&key_file.address.to_string())).unwrap();

        println!("No.{} wallet was generated.", i);
    }
}

pub fn sign_transaction<'a>(to: &str, value: &str, gas_limit: &str, data: &str) -> &'a str {
    let wallet = WALLETS.lock()
        .unwrap()
        .next()
        .unwrap();

    let from = wallet.file_name()
        .unwrap()
        .to_str()
        .unwrap();

    let balance = get_info(GET_BALANCE_API, from);
    let nonce = get_info(GET_TRANSACTION_COUNT_API, from);
    let value = value.as_bytes().to_hex();

    let private_key = {
        let mut key_file = String::new();
        File::open(wallet.clone())
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
        transaction.set_item(py, "gasPrice", "0x174876e800").unwrap();
        transaction.set_item(py, "nonce", nonce).unwrap();
        transaction.set_item(py, "data", data).unwrap();

        let web3 = web3.get(py, "Web3").unwrap();
        let from = web3.call_method(py, "toChecksumAddress", (from, ), None).unwrap();
        let to = web3.call_method(py, "toChecksumAddress", (to, ), None).unwrap();

        transaction.set_item(py, "to", to).unwrap();
        transaction.set_item(py, "value", value).unwrap();

        if gas_limit.is_empty() {
            transaction.set_item(py, "gas", "0x186a0").unwrap();
        } else {
            transaction.set_item(py, "gas", gas_limit).unwrap();
        }

        transaction
    };

    println!("{:?}", transaction.items(py));

    let account = web3.get(py, "Account").unwrap();
    let signed_transaction = account.call_method(py, "signTransaction", (transaction, private_key), None).unwrap();

    println!("{:?}", signed_transaction);
    unimplemented!()
}

pub fn transact(signed_transaction: &str) {
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
                println!("{}", resp.text().unwrap());
                break;
            }
            Err(e) => {
                println!("{:?}", e);
                continue;
            }
        }
    }
}

pub fn settle_accounts() {}

#[test]
fn test() {
    println!("{:x}", 100000000000000000000000u64);
//    sign_transaction("0xdce69e7f233b8876019093e0c8abf75e33dd8603", "100000000000000000", "", "");
}

//    for key_file_path in read_dir("wallets").unwrap() {
//        let path = key_file_path.unwrap().path();
//        if !path.file_name().unwrap().to_str().unwrap().starts_with("0x") { continue; }
//
//        let key_file = {
//            let mut data = String::new();
//            let mut key_file = File::open(path).unwrap();
//            key_file.read_to_string(&mut data);
//
//            KeyFile::decode(data).unwrap()
//        };
//
//        println!("{:?}", key_file.decrypt_key("123456789").unwrap().to_address().unwrap());
//    }