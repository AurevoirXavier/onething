// --- std ---
use std::{
    u128,
    io::{stdin, stdout, Write},
    thread::{self, sleep},
    time::Duration,
};

// --- custom ---
use crate::util::init::{CONF, GET_BALANCE_API, WALLETS};
use super::{
    format_balance_output,
    get_info,
    get_all_balance,
    get_premier_wallet,
    list_wallet,
    transact_core::{Transaction, check_balance},
};

pub fn sign_transaction_with_random_wallet<'a>(to: &'a str, value: &'a str, gas_limit: &'a str, data: &'a str) -> Transaction<'a> {
    let mut wallet;
    let mut from;
    loop {
        let mut guard = WALLETS.lock().unwrap();
        wallet = guard.next();

        from = wallet.file_name()
            .unwrap()
            .to_str()
            .unwrap();

        if check_balance(from, value, gas_limit) {
            return Transaction::new(to, value, gas_limit, data).sign(&wallet).to_owned();
        } else {
            guard.update(&wallet);
            continue;
        }
    }
}

pub fn dispatch_link_token() {
    let mut value = String::new();
    print!("Amount: ");
    stdout().flush().unwrap();
    stdin().read_line(&mut value).unwrap();

    let premier_wallet = get_premier_wallet();
    let from = premier_wallet.file_name().unwrap().to_str().unwrap();
    let value = (value.trim().parse::<f64>().unwrap() * 10f64.powi(18)).to_string();

    for (i, to) in list_wallet("wallets").into_iter().enumerate() {
        if check_balance(from, &value, "0x186a0") {
            let to = to.file_name().unwrap().to_str().unwrap();
            Transaction::new(to, &value, "0x186a0", "")
                .sign(&premier_wallet)
                .send();

            println!("[{}/50] Wallet [{}] -> Wallet [{}].", i, from, to);
        } else {
            println!("Wallet [{}]'s balance not enough.", from);
            break;
        }
    }

    sleep(Duration::from_secs(1));

    get_all_balance();
    println!("{}", format_balance_output(from));
}

pub fn collect_link_token() {
    let premier_wallet = get_premier_wallet().file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();

    let mut handles = vec![];
    for wallets in list_wallet("wallets").chunks(CONF.wallet_per_thread) {
        let premier_wallet = premier_wallet.clone();
        let wallets = wallets.to_vec();
        let handle = thread::spawn(move || {
            for wallet in wallets {
                let from = wallet.file_name().unwrap().to_str().unwrap();

                let remain = get_info(GET_BALANCE_API, from);
                if remain == "0x0" {
                    println!("Wallet [{}] already empty.", from);
                    continue;
                }

                let value = u128::from_str_radix(&remain[2..], 16).unwrap() - 0x2386f26fc10000;

                Transaction::new(&premier_wallet, &value.to_string(), "0x186a0", "")
                    .sign(&wallet)
                    .send();

                println!("Wallet [{}] -> Wallet [{}].", from, premier_wallet);
            }
        });

        handles.push(handle);
    }

    for handle in handles { handle.join().unwrap(); }

    sleep(Duration::from_secs(1));

    get_all_balance();
    println!("PREMIER {}", format_balance_output(&premier_wallet));
}

pub fn settle_accounts() {}  // TODO
