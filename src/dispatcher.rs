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
    },
    wallet::{
        gen_wallet,
        get_all_balance,
        transact::{collect_link_token, dispatch_link_token, settle_accounts},
        transact_core::Transaction,
    },
};

fn execute_task(t_id: u8, accounts: &[String], proxy: Option<&Arc<Mutex<Proxies>>>, kind: Option<u8>) {
    for account in accounts.iter() {
        let account: Vec<&str> = account.split('=').collect();
        let username = account[0];
        let password = account[1];

        println!("Account {} at {} thread.", username, t_id);  // TODO Verbose info

        match Account::new(username, password, proxy).sign_in(false) {
            Ok(account) => if let Some(kind) = kind { account.redeem(kind, false); } else { account.export(); }
            Err(e) => {
                println!("{}", e);  // TODO Debug
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
        "--balance" => get_all_balance(),
        "--collect" => collect_link_token(),
        "--export" => dispatch_account(None, with_proxy),
        "--dispatch" => dispatch_link_token(&args[2]),
        "--gen-wallet" => gen_wallet(),
        "--redeem" => Detector::new()
            .with_proxy()
            .with_kinds(&CONF.kinds)
            .detect(),
        "--settle" => settle_accounts(),  // TODO
        "--transact" => {
            let (gas_limit, data) = if args.len() == 7 { (args[5].as_str(), args[6].as_str()) } else { ("0x186a0", "") };
            Transaction::new(&args[3], &args[4], gas_limit, data)
                .sign(&PathBuf::from(&args[2]))
                .send();
        }
        _ => panic!("Unexpected args.")
    }
}
