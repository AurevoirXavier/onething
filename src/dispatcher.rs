// --- std ---
use std::{
    env,
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
};

// --- custom ---
use crate::{
    account::Account,
    detector::Detector,
    util::{
        init::{ACCOUNTS, CONF, ORDERS, PROXIES},
        proxy::Proxies,
        transaction::settle_accounts,
    },
    wallet::{gen_wallet, send_transaction, sign_transaction},
};

fn execute_task(t_id: u8, accounts: &[String], proxy: Option<&Arc<Mutex<Proxies>>>, kind: Option<u8>) {
    for account in accounts.iter() {
        let account: Vec<&str> = account.split('=').collect();
        let username = account[0];
        let password = account[1];

        println!("Account {} at {} thread.", username, t_id);

        match Account::new(username, password, proxy).sign_in(false) {
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
    for (i, accounts) in ACCOUNTS.chunks(CONF.account_per_thread).enumerate() {
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

    ORDERS.lock()
        .unwrap()
        .sync_all()
        .unwrap();
}

pub fn dispatch_task(with_proxy: bool) {
    let args: Vec<String> = env::args().collect();
    match args[1].as_str() {
        "--redeem" => Detector::new()
            .with_proxy()
            .with_kinds(&CONF.kinds)
            .detect(),
        "--export" => dispatch_account(None, with_proxy),
        "--settle" => settle_accounts(),  // TODO
        "--transact" => send_transaction(&sign_transaction(&PathBuf::from(&args[2]), &args[3], &args[4], &args[5], "")),
        "--gen-wallet" => gen_wallet(),
        _ => panic!("Unexpected args.")
    }
}
