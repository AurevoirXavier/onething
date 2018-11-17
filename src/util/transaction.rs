// --- std ---
use std::fs::{File, read_dir};

// --- custom ---
use crate::wallet::{send_transaction, sign_transaction};

pub fn dispatch_link_token(value: &str) {
    let premier_wallet = if let Some(premier_wallet) = read_dir(".").unwrap()
        .map(|d| d.unwrap())
        .find(|d| d.file_name().to_str().unwrap().starts_with("0x")) {
        premier_wallet.path()
    } else { panic!("Can find premier wallet."); };

//    println!("{:?}", premier_wallet);  // TODO Debug

    for to in read_dir("wallets").unwrap() {
        let to = to.unwrap().file_name();
        let to = to.to_str().unwrap();
        if !to.starts_with("0x") { continue; }

        send_transaction(&sign_transaction(&premier_wallet, to, value, "", ""));
    }
}

#[test]
fn test() {
    println!("{}", format!("{}{:0<18}", 100, 0));
}

pub fn settle_accounts() {}
