// --- std ---
use std::env;
use std::fs::{File, create_dir};
use std::io::prelude::*;
use std::io::Write;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;

// --- external ---
use uuid::Uuid;

// --- custom ---
use crate::{
    detector::Detector,
    account::{Account, proxy::Proxies},
};

lazy_static! {
    static ref ACCOUNTS: Vec<String> = {
        let mut f = File::open(Path::new("accounts.txt")).unwrap();
        let mut accounts = String::new();
        f.read_to_string(&mut accounts).unwrap();

        accounts.lines().map(|line| line.to_owned()).collect()
    };

    pub static ref PROXIES: Arc<Mutex<Proxies>> = Arc::new(Mutex::new(Proxies::new()));

    pub static ref ORDERS: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));
}

fn execute_task(t_id: u8, accounts: &[String], proxy: Option<&Arc<Mutex<Proxies>>>, kind: Option<u8>) {
    for account in accounts.iter() {
        let account: Vec<&str> = account.split('=').collect();
        let name = account[0];
        let pass = account[1];

        println!("Account {} at {} thread.", name, t_id);

        match Account::new(name, pass, proxy).sign_in(false) {
            Ok(account) => if let Some(kind) = kind { account.redeem(kind, false); } else { account.export(); }
            Err(e) => {
                println!("{}", e);
                continue;
            }
        }
    }
}

pub fn dispatch_account(kind: Option<u8>, with_proxy: bool) {
    let mut handles = vec![];
    for (i, accounts) in ACCOUNTS.chunks(5).enumerate() {
        let proxies = Arc::clone(&PROXIES);
        let handle = thread::spawn(move || {
            if with_proxy {
                execute_task(i as u8 + 1, accounts, Some(&proxies), kind);
            } else {
                execute_task(i as u8 + 1, accounts, None, kind);
            }
        });

        handles.push(handle);
    }

    for handle in handles { handle.join().unwrap(); }

    {
        let dir = Path::new("order");
        if !dir.exists() { create_dir(dir).unwrap(); }
    }
    let path = format!("order/orders-{}.txt", Uuid::new_v4());
    let mut f = File::create(Path::new(&path)).unwrap();
    f.write_all(
        ORDERS.lock()
            .unwrap()
            .join("\n")
            .as_bytes()
    ).unwrap();
    f.sync_all().unwrap();
}

pub fn dispatch_task(with_proxy: bool) {
    // TODO file not found
    match env::args().collect::<Vec<String>>()[1].as_str() {
        "--redeem" => Detector::new()
            .with_proxy()
            .with_kinds(&[1, 2, 3, 4, 5, 6])
            .detect(),
        "--export" => dispatch_account(None, with_proxy),
        _ => panic!("Unexpected args")
    }
}