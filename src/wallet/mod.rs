pub mod transact;
pub mod transact_core;

// --- std ---
use std::{
    collections::HashSet,
    fs::{create_dir, read_dir},
    io::{Write, stdin, stdout},
    iter::Cycle,
    path::{Path, PathBuf},
    thread,
    vec::IntoIter,
};

// --- external ---
use emerald_core::keystore::{KdfDepthLevel, KeyFile};
use serde_json::{Value, from_str};

// --- custom ---
use crate::util::{
    default_client_builder,
    format_balance,
    init::CONF,
};

pub struct Wallets {
    all: HashSet<PathBuf>,
    available: Cycle<IntoIter<PathBuf>>,
}

impl Wallets {
    pub fn new() -> Wallets {
        let all: Vec<PathBuf> = list_wallet("wallets");
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

    pub fn next(&mut self) -> PathBuf {
        if self.all.len() == 0 { panic!("All wallets are unavailable."); }
        self.available.next().unwrap()
    }
}

fn list_wallet(path: &str) -> Vec<PathBuf> {
    read_dir(path).unwrap()
        .map(|d| d.unwrap().path())
        .filter(|path| path.file_name().unwrap().to_str().unwrap().starts_with("0x"))
        .collect()
}

fn get_premier_wallet() -> PathBuf {
    let premier_wallet = list_wallet(".");
    if premier_wallet.is_empty() { panic!("Can find premier wallet."); } else { premier_wallet[0].to_owned() }
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
    let mut password = password.trim();
    if password.is_empty() { password = "123456789"; }

    {
        let dir = Path::new("new-wallets");
        if !dir.exists() { create_dir(dir).unwrap(); }
    }

    for i in 1..=amount.trim().parse::<u64>().unwrap() {
        let key_file = KeyFile::new(
            password,
            &KdfDepthLevel::Normal,
            None,
            None,
        ).unwrap();
        key_file.flush("new-wallets", Some(&key_file.address.to_string())).unwrap();

        println!("No.{} wallet was generated.", i);
    }
}

pub fn get_info(url: &str, address: &str) -> String {
    loop {
        if let Ok(mut resp) = default_client_builder(0)
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

pub fn get_all_balance() {
    let mut handles = vec![];
    for wallets in list_wallet("wallets").chunks(CONF.wallet_per_thread) {
        let wallets = wallets.to_owned();
        let handle = thread::spawn(move || {
            for wallet in wallets {
                println!("{}", format_balance(wallet.file_name().unwrap().to_str().unwrap()));
            }
        });

        handles.push(handle);
    }

    for handle in handles { handle.join().unwrap(); }
}
