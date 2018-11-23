pub mod init;
pub mod proxy;

// --- std ---
use std::{
    u128,
    fs::OpenOptions,
    io::Write,
    time::Duration,
};

// --- external ---
use reqwest::ClientBuilder;

// --- custom ---
use crate::wallet::get_info;
use self::init::{CODES, CONF, GET_BALANCE_API};

pub fn default_client_builder(timeout: u64) -> ClientBuilder {
    let timeout = if timeout == 0 { CONF.request_timeout } else { timeout };
    ClientBuilder::new()
        .danger_accept_invalid_certs(true)
        .danger_accept_invalid_hostnames(true)
        .gzip(true)
        .timeout(Duration::from_secs(timeout))
}

pub fn format_kind<'a>(kind: u8) -> &'a str {
    match kind {
        1 => "爱奇艺黄金会员12个月",
        2 => "爱奇艺VIP钻石会员年卡",
        3 => "爱奇艺黄金会员6个月",
        4 => "爱奇艺会员季卡",
        5 => "爱奇艺会员月卡",
        6 => "爱奇艺黄金会员周卡",
        7 => "爱奇艺钻石VIP会员",
        8 => "迅雷超级会员月卡",
        9 => "迅雷白金会员月卡",
        10 => "《链与消消乐》邀请码",
        _ => unreachable!()
    }
}

pub fn format_balance(address: &str) -> String {
    let balance = from_hex(&get_info(GET_BALANCE_API, address));
    format!("Wallet [{}] remains [{}] link token.", address, balance)
}

pub fn to_hex(decimal: &str) -> String { (decimal.parse::<f64>().unwrap() * 10f64.powi(18)).to_string() }

pub fn from_hex(hex: &str) -> f64 { u128::from_str_radix(&hex[2..], 16).unwrap() as f64 / 10f64.powi(18) }

pub fn save_export() {
    let mut kinds = [
        vec!["爱奇艺黄金会员12个月"],
        vec!["爱奇艺VIP钻石会员年卡"],
        vec!["爱奇艺黄金会员6个月"],
        vec!["爱奇艺会员季卡"],
        vec!["爱奇艺会员月卡"],
        vec!["爱奇艺黄金会员周卡"],
        vec!["爱奇艺钻石VIP会员"],
        vec!["迅雷超级会员月卡"],
        vec!["迅雷白金会员月卡"],
        vec!["《链与消消乐》邀请码"],
    ];

    let codes = CODES.lock().unwrap();
    for (kind, code) in codes.iter() {
        for kind_ in kinds.iter_mut() {
            if kind_[0] == kind { kind_.push(code); }
        }
    };

    let mut orders = OpenOptions::new()
        .append(true)
        .create(true)
        .open(format!("codes_{}.txt", CONF.date))
        .unwrap();

    for kind in kinds.iter() {
        if kind.len() == 1 { continue; }
        writeln!(orders, "{}\n{}\n", kind[0], kind[1..].join("\n")).unwrap();
    }
}
