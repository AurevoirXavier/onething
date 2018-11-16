// --- std ---
use std::fs::File;
use std::io::prelude::*;
use std::mem::swap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::vec::IntoIter;

// --- custom ---
use crate::{
    dispatcher::{PROXIES, dispatch_account},
    account::Account,
};

lazy_static! {
        static ref DETECTORS: Arc<Mutex<IntoIter<String>>> = {
            let mut f = File::open(Path::new("detectors.txt")).unwrap();
            let mut detectors = String::new();
            f.read_to_string(&mut detectors).unwrap();

            Arc::new(Mutex::new(detectors.lines().map(|line| line.to_owned()).collect::<Vec<String>>().into_iter()))
    };
}

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
        detector.sign_in(false).unwrap();

        let mut handles = vec![];
        for &kind in self.kinds.iter() {
            let proxy = self.proxy.clone();
            let mut detector = detector.clone();
            detector.session = detector.build_client();

            let handle = thread::spawn(move || {
                loop {
                    println!("Detecting {}.", kind);
                    match detector.redeem(kind, true) {
                        0 => {
                            println!("{} detected.", kind);
                            dispatch_account(Some(kind), proxy);
                            println!("{} detecting thread end.", kind);
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

                    thread::sleep(Duration::from_secs(1));
                }
            });

            handles.push(handle);
        }

        for handle in handles { handle.join().unwrap(); }
    }
}