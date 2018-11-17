// --- std ---
use std::{
    io::Write,
    thread::sleep,
    time::Duration,
};

// --- external ---
use serde_json::{Value, from_str};

// --- custom ---
use crate::{
    util::init::{ORDERS, SUBMIT_ORDER_API},
    wallet::{sign_transaction, transact},
};
use super::Account;

fn build_payload(kind: u8) -> Value {
    // payload.0 -> businessid
    // payload.1 -> exchange_price, price
    // payload.2 -> goodsid
    // payload.3 -> name
    let payload = match kind {
        1 => (3u8, 55., 101u8, "爱奇艺黄金会员12个月"),
        2 => (3, 140., 103, "爱奇艺VIP钻石会员年卡"),
        3 => (3, 30., 100, "爱奇艺黄金会员6个月"),
        4 => (3, 15., 13, "爱奇艺会员季卡"),
        5 => (3, 6., 12, "爱奇艺会员月卡"),
        6 => (3, 2., 99, "爱奇艺黄金会员周卡"),
        7 => (1, 6., 5, "迅雷超级会员月卡"),
        8 => (1, 4., 7, "迅雷白金会员月卡"),
        9 => (22, 0.1, 37, "《链与消消乐》邀请码"),
        _ => unreachable!()
    };

    return json!({
        "orders": [{
            "uid": "11111111-2222-3333-444444-555555",
            "businessid": payload.0,
            "num": 1,
            "exchange_price": payload.1,
            "goodsid": payload.2,
            "name": payload.3,
            "price": payload.1
        }]
    });
}

fn save_and_pay_order(account: &str, data: &Value) {
    let to = data["to"].as_str().unwrap();
    let value = data["value"].as_str().unwrap();
    let gas_limit = data["gas_limit"].as_u64().unwrap().to_string();
    let data = data["data"].as_str().unwrap();

    {
        let mut orders = ORDERS.lock().unwrap();
        writeln!(orders, "{}-{}-{}-{}-{}", account, to, value, gas_limit, data).unwrap();
    }

    transact(&sign_transaction(&gas_limit, to, value, data));
}

impl<'a> Account<'a> {
    pub fn redeem(&mut self, kind: u8, detect: bool) -> u8 {
        loop {
            let data;
            loop {
                let payload = build_payload(kind);
                data = match self.session.post(SUBMIT_ORDER_API)
                    .json(&payload)
                    .send() {
                    Ok(mut resp) => resp.text().unwrap(),
                    Err(e) => {
                        if e.is_server_error() || e.is_client_error() { continue; } else {
//                            println!("{:?}", e);  // TODO Debug
                            self.session = self.build_client();
                            continue;
                        }
                    }
                };

//                println!("{}", payload);
//                println!("{}", data);  // TODO Debug

                if data.contains('<') {
                    self.session = self.build_client();
                    return self.redeem(kind, false);
                }

                break;
            }

            let order: Value = from_str(&data).unwrap();
            println!("{}", order);  // TODO Debug
            if let Some(i_ret) = order.get("iRet") {
                match i_ret.as_i64() {
                    // iRet: -1, sMsg: 提交太频繁，请稍后再试
                    // iRet: -1, sMsg: 合约调用失败，请重试
                    Some(-1) => {
                        if detect { return 0; }

                        sleep(Duration::from_secs(1));
                        continue;
                    }
                    // iRet: 0, sMsg: 成功
                    Some(0) => {
                        save_and_pay_order(&self.username, &order["data"]);
                        return 0;
                    }
                    // iRet: 10090, sMsg: 您购买的产品已售空
                    Some(10090) => return 1,
                    // iRet: 10060, sMsg: 该商品每人限兑一次，去看看其他商品吧~
                    Some(10060) => return 7,
                    // iRet: 403, sMsg: 请登录后再操作
                    Some(403) => match self.sign_in(true) {
                        Ok(account) => return account.redeem(kind, false),
                        Err(e) => {
                            println!("{}", e);  // TODO Debug
                            return 1;
                        }
                    }
                    // TODO unhandled status code
                    Some(i_ret) => {
                        println!("Catch unhandled i_ret code {} in redeem!!\n{}", i_ret, order);  // TODO Debug
                        continue;
                    }
                    None => unreachable!()
                };
            } else { continue; }
        }
    }
}
