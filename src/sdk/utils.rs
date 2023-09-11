use std::str::FromStr;

use namada::types::{key::common::SecretKey, address::Address};

pub fn sk_from_str(sk: &str) -> SecretKey {
    SecretKey::from_str(sk).expect("Should be able to decode secret key.")
}

pub fn str_to_address(data: &str) -> Address {
    Address::from_str(data).expect("Should be able to decode address")
}