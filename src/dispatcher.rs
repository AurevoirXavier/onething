// --- std ---
use std::{
    env,
    path::PathBuf,
    sync::Mutex,
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
        init::{ACCOUNTS, CONF, ORDERS, PROXIES, TRANSACTION_THREADS},
        proxy::Proxies,
    },
    wallet::{
        gen_wallet,
        get_all_balance,
        transact::{collect_link_token, dispatch_link_token, settle_accounts},
        transact_core::Transaction,
    },
};

fn execute_task(t_id: u8, accounts: &[String], proxies: Option<&Mutex<Proxies>>, kind: Option<u8>) {
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
    {
        let mut handles = vec![];
        for (i, accounts) in ACCOUNTS.chunks(CONF.account_per_thread).enumerate() {
            let handle = if with_proxy {
                thread::spawn(move || { execute_task(i as u8 + 1, accounts, Some(&PROXIES), kind); })
            } else {
                thread::spawn(move || { execute_task(i as u8 + 1, accounts, None, kind); })
            };

            handles.push(handle);
        }

        for handle in handles { handle.join().unwrap(); }
    }

    {
        let mut transaction_threads = TRANSACTION_THREADS.lock().unwrap();
        while let Some(transaction_thread) = transaction_threads.pop() { transaction_thread.join().unwrap(); }
    }

    ORDERS.lock()
        .unwrap()
        .sync_all()
        .unwrap();
}

fn redeem() {
    let mut handles = vec![];
    for &kind in &CONF.kinds {
        let handle = thread::spawn(move || { dispatch_account(Some(kind), CONF.redeem_with_proxy); });
        handles.push(handle);
    }

    for handle in handles { handle.join().unwrap(); }
}

pub fn dispatch_task() {
    let args: Vec<String> = env::args().collect();
    match args[1].as_str() {
        "--balance" => if args.len() == 3 { println!("{}", format_balance(&args[2])); } else { get_all_balance(); }
        "--collect" => collect_link_token(),
        "--export" => {
            dispatch_account(None, CONF.export_with_proxy);
            save_export();
        }
        "--dispatch" => dispatch_link_token(),
        "--gen-wallet" => gen_wallet(),
        "--redeem" => if CONF.detect { Detector::new().with_proxy(CONF.redeem_with_proxy).with_kinds(&CONF.kinds).detect(); } else { redeem(); },
        "--settle" => settle_accounts(),
        "--transact" => {
            let (gas_limit, data) = if args.len() == 8 { (args[6].as_str(), args[7].as_str()) } else { ("0x186a0", "") };
            Transaction::new(&args[4], &to_hex(&args[5]), gas_limit, data)
                .sign(&PathBuf::from(&args[2]), &args[3])
                .send();
        }
        _ => panic!("Unexpected args.")
    }
}
