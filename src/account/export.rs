// --- std ---
//use std::fs::{File, create_dir};
//use std::io::prelude::*;
//use std::path::Path;

// --- external ---
use serde_json::{Value, from_str};

// --- custom ---
use crate::util::init::ORDER_LIST_API;
use super::Account;

impl<'a> Account<'a> {
    fn pull_order_list(&mut self, page: &str) -> bool {
        let data;
        loop {
            data = match self.session.post(ORDER_LIST_API)
                .form(&[
                    ("page", page),
                    ("status", "0")
                ]).send() {
                Ok(mut resp) => resp.text().unwrap(),
                Err(e) => {
                    println!("{:?}", e);
                    continue;
                }
            };

//            println!("{}", resp.status());  // TODO Debug
//            println!("{}", data);  // TODO Debug
            if data.contains('<') { return self.pull_order_list(page); }
            break;
        };

        let order_list: Value = from_str(&data).unwrap();
//        println!("{}", order_list);  // TODO Debug
        if let Some(i_ret) = order_list.get("iRet") {
            match i_ret.as_i64() {
                // iRet: 0, sMsg: 成功
                Some(0) => if order_list["data"]["next_page"].as_u64().unwrap() == 0 { true } else { false }
                // iRet: 403, sMsg: 请登录后再操作
                Some(403) => {
                    match self.sign_in(true) {
                        Ok(account) => account.export(),
                        Err(e) => println!("{}", e),
                    }

                    true
                }
                // TODO more status code
                Some(i_ret) => {
                    println!("Catch unhandled i_ret code {} in pull_order_list!!\n{}", i_ret, order_list);
                    true
                }
                _ => unreachable!()
            }
        } else { self.pull_order_list(page) }
    }

    pub fn export(&mut self) {
        for page in 0u8.. { if self.pull_order_list(&page.to_string()) { break; } }
    }
}
