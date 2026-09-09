//! Development-only local integrity helpers.

use std::io;

use serde::Serialize;

pub fn digest_json<T>(value: &T) -> io::Result<String>
where
    T: Serialize,
{
    let bytes = serde_json::to_vec(value).map_err(io::Error::other)?;
    Ok(digest_bytes(&bytes))
}

pub fn digest_bytes(bytes: &[u8]) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("fnv64:{hash:016x}")
}

pub fn sign_dev_placeholder(issued_by: &str, digest: &str) -> String {
    format!("dev-placeholder-signature:{issued_by}:{digest}")
}

pub fn dev_placeholder_issuer(issued_by: &str) -> String {
    format!("dev-placeholder:{issued_by}")
}
