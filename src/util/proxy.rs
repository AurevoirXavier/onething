// --- custom ---
use crate::{
    account::Account,
    util::{
        default_client_builder,
        init::CONF,
    },
};

pub struct Proxies(pub Vec<String>);

impl Proxies {
    pub fn new() -> Proxies { Proxies(vec![]) }

    pub fn update(&mut self, api: &str) {
        loop {
            if let Ok(mut resp) = default_client_builder()
                .build()
                .unwrap()
                .get(api)
                .send() {
                let data = resp.text().unwrap();
//                println!("{}", data);  // TODO Debug
                self.0 = data.lines().map(|line| line.to_owned()).collect();

                return;
            }
        }
    }
}

impl<'a> Account<'a> {
    pub fn ask_proxy(&self) -> String {
        if let Some(proxies) = self.proxies {
            let mut proxies = proxies.lock().unwrap();
            if proxies.0.is_empty() {
                proxies.update(&CONF.proxy_pool_api);
            }

            proxies.0.pop().unwrap()
        } else { String::new() }
    }
}
