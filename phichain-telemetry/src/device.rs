//! Device identification for telemetry.

use sha2::{Digest, Sha256};

/// Returns a stable, anonymous device identifier.
///
/// Computes a SHA-256 hash of the machine's unique ID to produce a
/// consistent identifier without exposing the raw machine ID. If the
/// machine ID cannot be read (e.g., in sandboxed environments), returns
/// `None` instead.
///
/// Returns `Some` containing a 64-character lowercase hex string (the
/// SHA-256 digest) when a stable machine ID is available.
pub fn get_device_id() -> Option<String> {
    let machine_id = machine_uid::get().ok()?;
    let mut hasher = Sha256::new();
    hasher.update(machine_id.as_bytes());
    let hash_result = hasher.finalize();
    Some(hash_result.iter().map(|b| format!("{b:02x}")).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn device_id_is_64_char_hex_when_available() {
        let id = get_device_id();
        if let Some(id) = id {
            assert_eq!(id.len(), 64);
            assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
        }
    }

    #[test]
    fn device_id_is_stable_when_available() {
        let id1 = get_device_id();
        let id2 = get_device_id();
        assert_eq!(id1, id2);
    }
}
