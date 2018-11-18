// --- std ---
use std::{
    mem::swap,
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
        let detector = DETECTORS.lock().unwrap().next().unwrap();
        let detector: Vec<&str> = detector.split('=').collect();
        let mut detector = Account::new(detector[0], detector[1], Some(&PROXIES));

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
                            break;
                        }
                        7 => if let Some(new_detector) = DETECTORS.lock().unwrap().next() {
                            let new_detector: Vec<&str> = new_detector.split('=').collect();
                            swap(
                                &mut detector.session,
                                &mut Account::new(new_detector[0], new_detector[1], Some(&PROXIES))
                                    .sign_in(false)
                                    .unwrap()
                                    .session,
                            );
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
