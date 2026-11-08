//! Round-robin API key pools, independent per provider.
//!
//! A [`KeyPool`] holds the already-resolved secret values for one
//! provider's enabled key slots (the caller — `jarvis-core::secrets` plus
//! the desktop app's settings — is responsible for loading them). It only
//! handles rotation and temporary cooldowns (e.g. after a 429); it never
//! persists anything and never logs a raw secret.

use std::time::{Duration, Instant};

/// One configured key slot (e.g. "k1".."k5" for a provider).
#[derive(Debug, Clone)]
pub struct ApiKeySlot {
    pub id: String,
    pub enabled: bool,
    secret: String,
    cooldown_until: Option<Instant>,
}

impl ApiKeySlot {
    pub fn new(id: impl Into<String>, secret: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            enabled: true,
            secret: secret.into(),
            cooldown_until: None,
        }
    }

    fn is_available(&self) -> bool {
        self.enabled
            && self
                .cooldown_until
                .map_or(true, |until| Instant::now() >= until)
    }
}

/// Round-robin pool of key slots for a single provider.
#[derive(Debug, Default)]
pub struct KeyPool {
    slots: Vec<ApiKeySlot>,
    next: usize,
}

impl KeyPool {
    pub fn new(slots: Vec<ApiKeySlot>) -> Self {
        Self { slots, next: 0 }
    }

    pub fn is_empty(&self) -> bool {
        self.slots.is_empty()
    }

    pub fn len(&self) -> usize {
        self.slots.len()
    }

    /// Advances the rotation and returns the next *available* key's
    /// `(id, secret)`, skipping disabled or cooling-down slots. Returns
    /// `None` only when every slot is unavailable (or the pool is empty).
    pub fn next_key(&mut self) -> Option<(String, String)> {
        let len = self.slots.len();
        if len == 0 {
            return None;
        }
        for _ in 0..len {
            let i = self.next;
            self.next = (self.next + 1) % len;
            let slot = &self.slots[i];
            if slot.is_available() {
                return Some((slot.id.clone(), slot.secret.clone()));
            }
        }
        None
    }

    pub fn set_enabled(&mut self, id: &str, enabled: bool) {
        if let Some(slot) = self.slots.iter_mut().find(|s| s.id == id) {
            slot.enabled = enabled;
        }
    }

    /// Puts a key on cooldown (e.g. after a 429 with a `Retry-After`),
    /// so rotation skips it until the cooldown elapses.
    pub fn mark_rate_limited(&mut self, id: &str, retry_after: Duration) {
        if let Some(slot) = self.slots.iter_mut().find(|s| s.id == id) {
            slot.cooldown_until = Some(Instant::now() + retry_after);
        }
    }

    pub fn ids(&self) -> Vec<String> {
        self.slots.iter().map(|s| s.id.clone()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pool_of(n: usize) -> KeyPool {
        KeyPool::new(
            (1..=n)
                .map(|i| ApiKeySlot::new(format!("k{i}"), format!("secret-{i}")))
                .collect(),
        )
    }

    #[test]
    fn rotates_k1_through_k5_and_wraps_to_k1() {
        let mut pool = pool_of(5);
        let sequence: Vec<String> = (0..7).map(|_| pool.next_key().unwrap().0).collect();
        assert_eq!(
            sequence,
            vec!["k1", "k2", "k3", "k4", "k5", "k1", "k2"],
            "expected round-robin K1 -> K5 -> K1, got {sequence:?}"
        );
    }

    #[test]
    fn skips_disabled_slots() {
        let mut pool = pool_of(3);
        pool.set_enabled("k2", false);
        let sequence: Vec<String> = (0..4).map(|_| pool.next_key().unwrap().0).collect();
        assert_eq!(sequence, vec!["k1", "k3", "k1", "k3"]);
    }

    #[test]
    fn skips_rate_limited_slots_until_cooldown_elapses() {
        let mut pool = pool_of(2);
        pool.mark_rate_limited("k1", Duration::from_secs(60));
        assert_eq!(pool.next_key().unwrap().0, "k2");
        assert_eq!(pool.next_key().unwrap().0, "k2");
    }

    #[test]
    fn returns_none_when_every_slot_is_unavailable() {
        let mut pool = pool_of(1);
        pool.set_enabled("k1", false);
        assert!(pool.next_key().is_none());
    }

    #[test]
    fn empty_pool_returns_none() {
        let mut pool = KeyPool::new(vec![]);
        assert!(pool.next_key().is_none());
        assert!(pool.is_empty());
    }
}
