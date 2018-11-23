// --- std ---
use std::{
    fs::File,
    io::Read,
    sync::Arc,
    thread::{self, sleep},
    time::Duration,
};

// --- custom ---
use crate::{
    account::Account,
    dispatcher::dispatch_account,
    util::init::PROXIES,
};

pub struct Detector {
    kinds: Vec<u8>,
    proxy: bool,
}

impl Detector {
    pub fn new() -> Detector {
        Detector {
            kinds: vec![],
            proxy: false,
        }
    }

    pub fn with_kinds(mut self, kinds: &[u8]) -> Self {
        self.kinds = kinds.to_owned();
        self
    }

    pub fn with_proxy(mut self, proxy: bool) -> Self {
        self.proxy = proxy;
        self
    }

    fn try_sign_in<'a>(detectors: &Vec<String>, index: &mut usize) -> Option<Account<'a>> {
        loop {
            if index.to_owned() == detectors.len() { return None; }

            let mut detector = Account::from_str(&detectors[index.to_owned()]);
            if let Err(_e) = detector.sign_in(false) {
//                println!("{}", _e);  // TODO Debug
                *index += 1;
                continue;
            }

            return Some(detector);
        }
    }

    pub fn detect(&mut self) {
        let proxy = self.proxy;

        let detectors = {
            let mut f = File::open("detectors.txt").unwrap();
            let mut detectors = String::new();
            f.read_to_string(&mut detectors).unwrap();

            Arc::new(detectors
                .lines()
                .map(|line| line.to_owned())
                .collect::<Vec<String>>())
        };


        let mut handles = vec![];
        for &kind in self.kinds.iter() {
            let mut index = 0;
            let mut detector = if let Some(detector) = Detector::try_sign_in(&detectors, &mut index) { detector.with_proxies(&PROXIES) } else { continue; };
            detector.session = detector.build_client();

            let detectors = Arc::clone(&detectors);
            let handle = thread::spawn(move || {
                loop {
                    println!("Detecting [{}].", kind);

                    match detector.redeem(kind, true) {
                        0 => {
                            println!("[{}] detected.", kind);
                            dispatch_account(Some(kind), proxy);
                            index += 1;
                            println!("[{}] detecting thread end.", kind);
                        }
                        7 => detector = {
                            index += 1;
                            if let Some(detector) = Detector::try_sign_in(&detectors, &mut index) { detector.with_proxies(&PROXIES) } else { break; }
                        },
                        _ => ()
                    }

                    sleep(Duration::from_secs(1));
                }
            });

            handles.push(handle);
        }

        for handle in handles { handle.join().unwrap(); }
    }
}
