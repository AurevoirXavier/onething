// --- std ---
use std::fs::read_dir;

// --- custom ---
use crate::util::init::WALLETS;
use super::transact_core::{check_balance, send_transaction, sign_transaction};

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

        if check_balance(from, value, gas_limit) {
            return sign_transaction(&wallet, to, value, gas_limit, data);
        } else {
            guard.update(&wallet);
            continue;
        }
    }
}

pub fn dispatch_link_token(value: &str) {
    let premier_wallet = if let Some(premier_wallet) = read_dir(".").unwrap()
        .map(|d| d.unwrap())
        .find(|d| d.file_name().to_str().unwrap().starts_with("0x")) {
        premier_wallet.path()
    } else { panic!("Can find premier wallet."); };

//    println!("{:?}", premier_wallet);  // TODO Debug

    let value = (value.parse::<f64>().unwrap() * 10f64.powi(18)).to_string();

    for to in read_dir("wallets").unwrap() {
        let to = to.unwrap().file_name();
        let to = to.to_str().unwrap();

        if to.starts_with("0x")
            &&
            check_balance(premier_wallet.file_name().unwrap().to_str().unwrap(), &value, "0x186a0") {
            send_transaction(&sign_transaction(
                &premier_wallet,
                to,
                &value,
                "0x186a0",
                "",
            ));
        } else { continue; }
    }
}

pub fn settle_accounts() {}

#[test]
fn test() {
    send_transaction(&sign_transaction_with_random_wallet(
        "0xdce69e7f233b8876019093e0c8abf75e33dd8603",
        "100000000000000000",
        "0x186a0",
        "",
    ));
}