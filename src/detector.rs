// --- std ---
use std::{
    fs::File,
    io::Read,
    thread::{self, sleep},
    time::Duration,
};

// --- custom ---
use crate::{
    account::Account,
    dispatcher::dispatch_account,
    util::init::{DETECTORS, PROXIES},
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

    pub fn with_kinds(&mut self, kinds: &[u8]) -> &mut Self {
        self.kinds = kinds.to_owned();
        self
    }

    pub fn with_proxy(&mut self) -> &mut Self {
        self.proxy = true;
        self
    }

    pub fn detect(&mut self) {
//        let detectors = {
//            let mut f = File::open("detectors.txt").unwrap();
//            let mut detectors = String::new();
//            f.read_to_string(&mut detectors).unwrap();
//
//            detectors
//                .lines()
//                .map(|line| line.to_owned())
//                .collect::<Vec<String>>()
//        };

        let detector = DETECTORS.lock().unwrap().next().unwrap();
        let mut detector = Account::from_str(&detector).with_proxies(&PROXIES);

        if let Err(e) = detector.sign_in(true) {
            println!("{}", e);  // TODO Debug
            return;
        }

        let mut handles = vec![];
        for &kind in self.kinds.iter() {
            let proxy = self.proxy.clone();
            let mut detector = detector.clone();
            detector.session = detector.build_client();

            let handle = thread::spawn(move || {
                loop {
                    println!("Detecting [{}].", kind);
                    match detector.redeem(kind, true) {
                        0 => {
                            println!("[{}] detected.", kind);
                            dispatch_account(Some(kind), proxy);
                            println!("[{}] detecting thread end.", kind);
                        }
                        7 => if let Some(new_detector) = DETECTORS.lock().unwrap().next() {
                            detector = Account::from_str(&new_detector).with_proxies(&PROXIES);
                            if let Err(e) = detector.sign_in(true) {
                                println!("{}", e);  // TODO Debug
                                continue;
                            }
                        } else { break; }
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
