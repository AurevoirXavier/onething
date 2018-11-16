use super::Account;

const PROXY_POOL_API: &'static str = "http://api3.xiguadaili.com/ip/?tid=555538032022318&num=1000&category=2&delay=1";

pub struct Proxies(pub Vec<String>);

impl Proxies {
    pub fn new() -> Proxies { Proxies(vec![]) }

    pub fn update(&mut self, api: &str) {
        loop {
            if let Ok(mut resp) = reqwest::get(api) {
                self.0 = resp.text()
                    .unwrap()
                    .lines()
                    .map(|line| line.to_owned())
                    .collect();

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
                proxies.update(PROXY_POOL_API);
            }

            proxies.0.pop().unwrap()
        } else { String::new() }
    }
}