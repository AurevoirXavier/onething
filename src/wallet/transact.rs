// --- custom ---
use crate::util::init::WALLETS;
use super::{
    list_wallet,
    transact_core::{check_balance, send_transaction, sign_transaction},
};

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
    let premier_wallet = {
        let premier_wallet = list_wallet(".");
        if premier_wallet.is_empty() { panic!("Can find premier wallet."); } else { premier_wallet[0].to_owned() }
    };

    let from = premier_wallet.file_name().unwrap().to_str().unwrap();
    let value = (value.parse::<f64>().unwrap() * 10f64.powi(18)).to_string();

    for to in list_wallet("wallets") {
        if check_balance(from, &value, "0x186a0") {
            send_transaction(&sign_transaction(
                &premier_wallet,
                to.file_name().unwrap().to_str().unwrap(),
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