// --- std ---
use std::{
    collections::HashSet,
    fs::{File, create_dir, read_dir},
    io::{Read, Write, stdin, stdout},
    path::Path,
};

// --- external ---
use emerald_core::keystore::{KdfDepthLevel, KeyFile};

pub fn gen_wallet() {
    let mut amount = String::new();
    print!("Amount: ");
    stdout().flush().unwrap();
    stdin().read_line(&mut amount).unwrap();

    let mut password = String::new();
    print!("Password (Enter for default password `123456789`): ");
    stdout().flush().unwrap();
    stdin().read_line(&mut password).unwrap();
    if password.trim().is_empty() { password = "123456789".to_string(); }

    {
        let dir = Path::new("new-wallets");
        if !dir.exists() { create_dir(dir).unwrap(); }
    }
    for i in 1..=amount.trim().parse::<u64>().unwrap() {
        let key_file = KeyFile::new(
            &password,
            &KdfDepthLevel::Normal,
            None,
            None,
        ).unwrap();
        key_file.flush(Path::new("new-wallets"), Some(&key_file.address.to_string())).unwrap();

        println!("No.{} wallet was generated.", i);
    }
}

pub fn transact() {}

pub fn settle_accounts() {}

#[test]
fn test() {
    for key_file_path in read_dir("wallets").unwrap() {
        let path = key_file_path.unwrap().path();
        if !path.file_name().unwrap().to_str().unwrap().starts_with("0x") { continue; }

        let key_file = {
            let mut data = String::new();
            let mut key_file = File::open(path).unwrap();
            key_file.read_to_string(&mut data);

            KeyFile::decode(data).unwrap()
        };

        println!("{:?}", key_file.decrypt_key("123456789").unwrap());
    }
}
