extern crate chrono;
extern crate cpython;
extern crate emerald_core;
#[macro_use]
extern crate lazy_static;
extern crate md5;
extern crate rand;
extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate uuid;

mod account;
mod detector;
mod dispatcher;
mod util;
mod wallet;

// --- custom ---
use self::dispatcher::dispatch_task;

fn main() { dispatch_task(); }
