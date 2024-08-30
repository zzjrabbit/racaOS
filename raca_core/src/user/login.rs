use core::hash::Hasher;

use alloc::{format, string::String};
use rs_sha3_512::{HasherContext, Sha3_512Hasher};

pub fn try_pass_word(password: String, sha: String) -> bool {
    let mut key = Sha3_512Hasher::default();
    key.write(password.as_bytes());

    let byte_result = HasherContext::finish(&mut key);

    let result = format!("{byte_result:02X}");

    if result == sha {
        true
    } else {
        false
    }
}
