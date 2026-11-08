//! Secure storage for API keys and other credentials.
//!
//! Backed by the OS credential store via the `keyring` crate (Windows
//! Credential Manager, macOS Keychain, Linux Secret Service). Nothing in
//! this module writes a secret to disk, to `.env`, or to a log line —
//! callers must not log the value returned by [`get`].
//!
//! Entries are addressed by `(service, account)`, e.g.
//! `("jarvis.openrouter", "k1")` for the first OpenRouter key slot.

use crate::error::{CoreError, CoreResult};

const SERVICE_PREFIX: &str = "jarvis";

fn entry(service: &str, account: &str) -> CoreResult<keyring::Entry> {
    let service = format!("{SERVICE_PREFIX}.{service}");
    keyring::Entry::new(&service, account)
        .map_err(|e| CoreError::Secret(format!("could not open credential entry: {e}")))
}

/// Stores `value` under `(service, account)`, overwriting any existing value.
pub fn set(service: &str, account: &str, value: &str) -> CoreResult<()> {
    entry(service, account)?
        .set_password(value)
        .map_err(|e| CoreError::Secret(format!("could not store credential: {e}")))
}

/// Reads the value stored under `(service, account)`, if any.
///
/// Returns `Ok(None)` when no credential is stored yet, which is the
/// normal state before a user has configured a key — not an error.
pub fn get(service: &str, account: &str) -> CoreResult<Option<String>> {
    match entry(service, account)?.get_password() {
        Ok(value) => Ok(Some(value)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(CoreError::Secret(format!(
            "could not read credential: {e}"
        ))),
    }
}

/// Removes the value stored under `(service, account)`, if any.
pub fn delete(service: &str, account: &str) -> CoreResult<()> {
    match entry(service, account)?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(CoreError::Secret(format!(
            "could not delete credential: {e}"
        ))),
    }
}

/// Masks a secret for display/logging: keeps the first and last two
/// characters (if long enough) and replaces the middle with `...`.
/// Never log or display a raw key — use this everywhere a key must be
/// shown to the user (e.g. "already configured: sk-...xy").
pub fn mask(value: &str) -> String {
    let len = value.chars().count();
    if len <= 6 {
        return "*".repeat(len);
    }
    let chars: Vec<char> = value.chars().collect();
    let head: String = chars[..2].iter().collect();
    let tail: String = chars[len - 2..].iter().collect();
    format!("{head}...{tail}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mask_keeps_only_edges() {
        assert_eq!(mask("sk-or-v1-abcdefgh"), "sk...gh");
        assert_eq!(mask("short"), "*****");
        assert_eq!(mask(""), "");
    }
}
