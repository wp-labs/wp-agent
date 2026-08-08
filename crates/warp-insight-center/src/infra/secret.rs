// 密钥工具：sha256（与 warp-gateway 一致的实现）。

use ring::digest;

pub fn sha256_hex(value: &str) -> String {
    hex_lower(digest::digest(&digest::SHA256, value.as_bytes()).as_ref())
}

fn hex_lower(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push_str(&format!("{byte:02x}"));
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_hex_matches_known_vector() {
        assert_eq!(
            sha256_hex("abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
