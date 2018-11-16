// --- std ---
use std::fmt;
use std::fs::{File, create_dir};
use std::io::prelude::*;
use std::path::Path;
use std::time::Duration;

// --- external ---
use rand::Rng;
use reqwest::{Client, Proxy};
use reqwest::header::{COOKIE, SET_COOKIE, HeaderMap};
use serde_json::{Value, from_str};

// --- custom ---
use super::Account;

const SIGN_IN_API: &'static str = "https://api-accw.onethingpcs.com/user/login";

#[derive(Debug)]
pub enum SignInError {
    AccountWrong,
    //    SignatureWrong,  // TODO already handled
    UnHandle,
}

impl fmt::Display for SignInError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SignInError::AccountWrong => write!(f, "Fail to sign in, account or password is wrong!"),
//            SignInError::SignatureWrong => write!(f, "Fail to sign in, signature is wrong!"),  // TODO already handled
            SignInError::UnHandle => write!(f, "Fail to sign in, unknown error!")
        }
    }
}

impl<'a> Account<'a> {
    fn build_payload(username: &str, pass: &[u8]) -> Vec<(String, String)> {
        let cipher = format!("{:x}", md5::compute(pass));
        let mut tr = rand::thread_rng();
        let id = format!("{}b24fdf96f3c1ea236b849dbf936120ccef24b9fda1", tr.gen_range(100000, 999999));
        let uuid = format!("62B4A12E-A211-4EFD-A874-D80A29E{}C", tr.gen_range(1000, 9999));
        let pwd = format!(
            "{:x}",
            md5::compute([
                &cipher[0..2],
                &cipher[8..9],
                &cipher[3..8],
                &cipher[2..3],
                &cipher[9..17],
                &cipher[27..28],
                &cipher[18..27],
                &cipher[17..18],
                &cipher[28..]
            ].join(""))
        );
        let mut payload = vec![
            ("deviceid".to_string(), id.clone()),
            ("imeiid".to_string(), id),
            ("nc".to_string(), "GB".to_string()),
            ("ph_model".to_string(), "iPhone 6s".to_string()),
            ("ph_ver".to_string(), "iOS 12.0.1".to_string()),
            ("pwd".to_string(), pwd.to_owned()),
            ("uuid".to_string(), uuid.to_owned())
        ];

        if username.contains('@') {
            payload.push(("account_type".to_string(), "5".to_string()));
            payload.push(("mail".to_string(), username.to_owned()));
        } else {
            payload.push(("account_type".to_string(), "4".to_string()));
            payload.push(("phone".to_string(), username.to_owned()));
            payload.push(("phone_area".to_string(), "86".to_string()));
        }

        payload
    }

    fn generate_sign(mut payload: Vec<(String, String)>, key: &str) -> String {
        let mut items = vec![];
        while payload.len() != 0 {
            let item = payload.pop().unwrap();
            items.push(format!("{}={}", item.0, item.1));
        }
        items.sort();

        let mut i = 0;
        let mut sign = String::new();
        while i != items.len() {
            sign = format!("{}{}&", sign, items[i]);
            i += 1;
        }

        format!("{:x}", md5::compute(format!("{}key={}", sign, key).as_bytes()))
    }

    pub fn build_client(&self) -> Client {
        let client_builder = Client::builder()
            .danger_accept_invalid_certs(true)
            .danger_accept_invalid_hostnames(true)
            .default_headers(self.cookie.clone())
            .gzip(true)
            .timeout(Duration::from_secs(5));

        let proxy = self.ask_proxy();
        if proxy.is_empty() {
            client_builder.build().unwrap()
        } else {
            println!("Account: {}, with proxy: {}.", self.name, proxy);  // TODO Debug
//            println!("{}", self.proxies.unwrap().lock().unwrap().0.len());  // TODO Debug

            let proxy = format!("http://{}", proxy);
            client_builder.proxy(Proxy::https(&proxy).unwrap()).build().unwrap()
        }
    }

    fn save_cookie(&self, cookie: &[u8]) {
        {
            let dir = Path::new("cookie");
            if !dir.exists() { create_dir(dir).unwrap(); }
        }

        let path = format!("cookie/{}", self.name);
        let path = Path::new(&path);
        let mut f = File::create(path).unwrap();

        f.write_all(cookie).unwrap();
        f.sync_all().unwrap();
    }

    fn load_cookie_from_headers(&mut self, headers: &HeaderMap) {
        let mut cookie = vec![];
        for (k, v) in headers.iter() {
            if k == SET_COOKIE { cookie.push(v.to_str().unwrap()); }
        }

        let cookie = cookie.join("; ");
        self.save_cookie(cookie.as_bytes());

        self.cookie.insert(COOKIE, cookie.parse().unwrap());
        self.session = self.build_client();
    }

    fn load_cookie_from_file(&mut self, path: &Path) {
        let mut f = File::open(path).unwrap();
        let mut cookie = String::new();
        f.read_to_string(&mut cookie).unwrap();

        self.cookie.insert(COOKIE, cookie.parse().unwrap());
        self.session = self.build_client();
    }

    pub fn sign_in(&mut self, retry: bool) -> Result<&mut Account<'a>, SignInError> {
        if !retry {
            let path = format!("cookie/{}", self.name);
            let path = Path::new(&path);
            if path.exists() {
                self.load_cookie_from_file(path);
                return Ok(self);
            }
        }

        let mut payload = Account::build_payload(&self.name, self.pass.as_bytes());
        let sign = Account::generate_sign(payload.clone(), "");
        payload.push(("sign".to_string(), sign));

        loop {
            match Client::builder()
                .danger_accept_invalid_certs(true)
                .danger_accept_invalid_hostnames(true)
                .gzip(true)
                .timeout(Duration::from_secs(2))
                .build()
                .unwrap()
                .post(SIGN_IN_API)
                .form(payload.as_slice())
                .send() {
                Ok(mut resp) => {
                    let result: Value = from_str(&resp.text().unwrap()).unwrap();
//                    println!("{}", result);  // TODO Debug

                    if let Some(i_ret) = result.get("iRet") {
                        match i_ret.as_i64() {
                            // iRet: 0, sMsg: Success
                            Some(0) => {
                                self.load_cookie_from_headers(resp.headers());
                                return Ok(self);
                            }
                            // iRet: -129, sMsg: Incorrect account or password. Please enter again
                            Some(-129) => return Err(SignInError::AccountWrong),
                            // iRet: -109, sMsg: Signature error
//                            Some(-109) => return Err(SignInError::SignatureWrong),  // TODO already handled
                            _ => return Err(SignInError::UnHandle)
                        }
                    }
                }
                Err(e) => println!("{:?}", e)  // TODO Debug
            }
        }
    }
}