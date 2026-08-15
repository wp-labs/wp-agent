// 密钥工具：sha256 / 随机 token / RegistToken 派生（与 warp-gateway 一致的实现）。

use ring::{digest, hmac, rand as ring_rand};

pub fn sha256_hex(value: &str) -> String {
    hex_lower(digest::digest(&digest::SHA256, value.as_bytes()).as_ref())
}

/// 生成 32 字节随机 secret token：`"{prefix}_{hex}"`（镜像 warp-gateway）。
pub fn new_secret_token(prefix: &str) -> Result<String, String> {
    let mut bytes = [0_u8; 32];
    let rng = ring_rand::SystemRandom::new();
    ring_rand::SecureRandom::fill(&rng, &mut bytes)
        .map_err(|_| "failed to read system random source".to_string())?;
    Ok(format!("{prefix}_{}", hex_lower(&bytes)))
}

/// 由网关身份 Token（identity_token）派生一次性注册凭据 RegistToken：
/// `HMAC-SHA256(center_secret, "gateway-reg:{gateway_id}:{identity_token}")` 的 hex。
/// 派生确定性 + 网关/身份/密钥三重绑定；中心只存结果 hash，不重算。
pub fn derive_regist_token(secret: &str, gateway_id: &str, identity_token: &str) -> String {
    let key = hmac::Key::new(hmac::HMAC_SHA256, secret.as_bytes());
    let mut data = String::with_capacity(
        secret.len() + gateway_id.len() + identity_token.len() + "gateway-reg::".len(),
    );
    data.push_str("gateway-reg:");
    data.push_str(gateway_id);
    data.push(':');
    data.push_str(identity_token);
    hex_lower(hmac::sign(&key, data.as_bytes()).as_ref())
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

    #[test]
    fn new_secret_token_has_prefix_and_is_unique() {
        let token = new_secret_token("boot").expect("token");
        assert!(token.starts_with("boot_"), "prefix mismatch: {token}");
        assert_eq!(token.len(), "boot_".len() + 64, "expected 32 bytes hex");
        assert_ne!(token, new_secret_token("boot").expect("token2"));
        assert_ne!(
            new_secret_token("boot").expect("a"),
            new_secret_token("reg").expect("b")
        );
    }

    #[test]
    fn derive_regist_token_matches_known_vector() {
        assert_eq!(
            derive_regist_token("center-secret", "gw-001", "identity-1"),
            "61b5cd8d58a745651723f144e06fd83f2aac9b8071c188b12108fde5f6c48c02"
        );
    }

    #[test]
    fn derive_regist_token_is_deterministic_and_bound() {
        let base = derive_regist_token("sec", "gw-001", "id-1");
        assert_eq!(base, derive_regist_token("sec", "gw-001", "id-1"));
        // 三重绑定：换密钥 / 换网关 / 换身份 → 均不同。
        assert_ne!(base, derive_regist_token("other", "gw-001", "id-1"));
        assert_ne!(base, derive_regist_token("sec", "gw-002", "id-1"));
        assert_ne!(base, derive_regist_token("sec", "gw-001", "id-2"));
    }
}
