// --- std ---
use std::{
    io::Write,
    thread::sleep,
    time::Duration,
};

// --- external ---
use serde_json::{Value, from_str};

// --- custom ---
use crate::util::init::{CODES, CONF, ORDER_INFO_API, ORDER_LIST_API};
use super::Account;

impl<'a> Account<'a> {
    fn pull_order(&mut self, order_id: &str) {
        self.session = self.build_client();

        let data;
        loop {
            data = match self.session
                .post(ORDER_INFO_API)
                .form(&[("order_id", order_id)])
                .send() {
                Ok(mut resp) => resp.text().unwrap(),
                Err(e) => {
//                    println!("{:?}", e);
                    self.session = self.build_client();
                    continue;
                }
            };

//            println!("{}", data);  // TODO Debug

            if data.contains('<') { return self.pull_order(order_id); }
            break;
        }

        let order: Value = from_str(&data).unwrap();
//        println!("{}", order_list);  // TODO Debug
        if let Some(i_ret) = order.get("iRet") {
            match i_ret.as_i64() {
                // iRet: -1, sMsg: 操作太频繁，请稍后重试
                Some(-1) => {
                    sleep(Duration::from_secs(1));
                    self.pull_order(order_id);
                }
                // iRet: 0, sMsg: 成功
                Some(0) => {
                    let info = &order["data"]["lists"][0];

                    // Transaction not finished.
                    if info["pay_status"].as_u64().unwrap() != 2 { return; }

                    let goods_name = info["goods_name"].as_str().unwrap();
                    let code = info["code"].as_str().unwrap();

                    let info = format!("[{}] -> [{}]", goods_name, code);
                    println!("{}", info);
                    writeln!(CODES.lock().unwrap(), "{}", info).unwrap();
                }
                // iRet: 403, sMsg: 请登录后再操作
                Some(403) => {
                    match self.sign_in(true) {
                        Ok(account) => account.export(),
                        Err(e) => println!("{}", e),  // TODO Debug
                    }
                }
                // Unhandled status code
                Some(i_ret) => println!("Catch unhandled i_ret code [{}] in pull_order!!\n{}", i_ret, order),
                _ => unreachable!()
            }
        } else { self.pull_order(order_id); }
    }

    fn pull_order_list(&mut self, page: &str) -> bool {
        self.session = self.build_client();

        let data;
        loop {
            data = match self.session
                .post(ORDER_LIST_API)
                .form(&[
                    ("page", page),
                    ("status", "0")
                ]).send() {
                Ok(mut resp) => resp.text().unwrap(),
                Err(e) => {
//                    println!("{:?}", e);
                    self.session = self.build_client();
                    continue;
                }
            };

//            println!("{}", data);  // TODO Debug

            if data.contains('<') { return self.pull_order_list(page); }
            break;
        };

        let order_list: Value = from_str(&data).unwrap();
//        println!("{}", order_list);  // TODO Debug
        if let Some(i_ret) = order_list.get("iRet") {
            match i_ret.as_i64() {
                // iRet: -1, sMsg: 操作太频繁，请稍后重试
                Some(-1) => {
                    sleep(Duration::from_secs(1));
                    self.pull_order_list(page)
                }
                // iRet: 0, sMsg: 成功
                Some(0) => {
                    let data = &order_list["data"];
                    if data["cur_page"] == data["next_page"] { return false; }

                    for lists in data["lists"].as_array().unwrap() {
                        let order_id = &lists["order_id"];
//                        println!("{}", order_id);  // TODO Debug

                        let order_id = order_id.as_str().unwrap();
                        if order_id[1..9] != CONF.date { return true; }

                        self.pull_order(order_id);
                    }

                    if data["next_page"].as_u64().unwrap() == 0 { true } else { false }
                }
                // iRet: 403, sMsg: 请登录后再操作
                Some(403) => {
                    match self.sign_in(true) {
                        Ok(account) => account.export(),
                        Err(e) => println!("{}", e),  // TODO Debug
                    }

                    true
                }
                // Unhandled status code
                Some(i_ret) => {
                    println!("Catch unhandled i_ret code [{}] in pull_order_list!!\n{}", i_ret, order_list);
                    true
                }
                _ => unreachable!()
            }
        } else { self.pull_order_list(page) }
    }

    pub fn export(&mut self) { for page in 0u8.. { if self.pull_order_list(&page.to_string()) { break; } } }
}
