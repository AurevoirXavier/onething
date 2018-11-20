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
        to_hex,
        format_balance,
        save_export,
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

fn execute_task(t_id: u8, accounts: &[String], proxies: Option<&Arc<Mutex<Proxies>>>, kind: Option<u8>) {
    for account in accounts.iter() {
        let account: Vec<&str> = account.split('=').collect();
        let username = account[0];

        println!("Account [{}] at [{}] thread.", username, t_id);

        match Account::new(username, account[1], proxies).sign_in(false) {
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
        let handle = if with_proxy {
            let proxies = Arc::clone(&PROXIES);
            thread::spawn(move || { execute_task(i as u8 + 1, accounts, Some(&proxies), kind); })
        } else {
            thread::spawn(move || { execute_task(i as u8 + 1, accounts, None, kind); })
        };

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
        "--balance" => if args.len() == 3 { println!("{}", format_balance(&args[2])); } else { get_all_balance(); }
        "--collect" => collect_link_token(),
        "--export" => {
            dispatch_account(None, with_proxy);
            save_export();
        }
        "--dispatch" => dispatch_link_token(),
        "--format-code" => save_export(),
        "--gen-wallet" => gen_wallet(),
        "--redeem" => Detector::new()
            .with_proxy()
            .with_kinds(&CONF.kinds)
            .detect(),
        "--settle" => settle_accounts(),
        "--transact" => {
            let (gas_limit, data) = if args.len() == 7 { (args[5].as_str(), args[6].as_str()) } else { ("0x186a0", "") };
            Transaction::new(&args[3], &to_hex(&args[4]), gas_limit, data)
                .sign(&PathBuf::from(&args[2]))
                .send();
        }
        _ => panic!("Unexpected args.")
    }
}
