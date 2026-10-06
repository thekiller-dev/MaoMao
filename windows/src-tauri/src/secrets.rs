// API keys live in the Windows Credential Manager or, on Linux, the Secret
// Service (GNOME Keyring, KWallet) — never on disk and never in the front end — the island can only ask whether a key is present.

use keyring::Entry;

const SERVICE: &str = "app.maomao";

/// Every key MaoMao may store. Anything outside this list is refused.
pub const KNOWN_KEYS: &[&str] = &[
    "anthropic-api-key",
    "n8n-url",
    "n8n-api-key",
    "vercel-token",
    "github-token",
    "stripe-api-key",
    "resend-api-key",
    "notion-api-key",
    "calcom-api-key",
];

/// Custom integrations use one key per generated integration ID. Keep the
/// namespace narrow: the renderer may ask to store a custom secret, but it must
/// never be able to turn this command into an arbitrary Credential Manager or
/// Secret Service lookup.
fn is_allowed_key(key: &str) -> bool {
    if KNOWN_KEYS.contains(&key) {
        return true;
    }
    let Some(id) = key.strip_prefix("custom-").and_then(|v| v.strip_suffix("-key")) else {
        return false;
    };
    let bytes = id.as_bytes();
    id.starts_with("custom_")
        && ("custom_".len()..=128).contains(&id.len())
        && bytes.iter().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || *b == b'_' || *b == b'-')
}

fn entry(key: &str) -> Option<Entry> {
    if !is_allowed_key(key) {
        return None;
    }
    Entry::new(SERVICE, key).ok()
}

pub fn get(key: &str) -> Option<String> {
    entry(key)?.get_password().ok().filter(|v| !v.is_empty())
}

pub fn set(key: &str, value: &str) -> Result<(), String> {
    let entry = entry(key).ok_or_else(|| format!("unknown key {key}"))?;
    if value.is_empty() {
        let _ = entry.delete_credential();
        return Ok(());
    }
    entry.set_password(value).map_err(|e| e.to_string())
}

pub fn clear(key: &str) -> Result<(), String> {
    let entry = entry(key).ok_or_else(|| format!("unknown key {key}"))?;
    match entry.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

pub fn present(key: &str) -> bool {
    get(key).is_some()
}

#[cfg(test)]
mod tests {
    use super::is_allowed_key;

    #[test]
    fn allows_declared_and_generated_custom_keys_only() {
        assert!(is_allowed_key("anthropic-api-key"));
        assert!(is_allowed_key("custom-custom_my_api_1234-key"));
        assert!(!is_allowed_key("custom-my_api-key"));
        assert!(!is_allowed_key("custom-custom_MyApi-key"));
        assert!(!is_allowed_key("custom-custom_api-key-extra"));
        assert!(!is_allowed_key("../../secret"));
    }

    #[test]
    fn rejects_an_oversized_custom_key() {
        let id = format!("custom-{}-key", "custom_".to_owned() + &"a".repeat(122));
        assert!(!is_allowed_key(&id));
    }
}
