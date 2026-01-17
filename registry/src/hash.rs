//! Hashing utilities for SKILL.md integrity verification

use sha2::{Digest, Sha256};

/// Compute SHA256 hash of skill content
pub fn compute_skill_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

/// Verify that content matches expected hash
pub fn verify_skill_hash(content: &str, expected_hash: &str) -> bool {
    let computed = compute_skill_hash(content);
    computed.eq_ignore_ascii_case(expected_hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_hash() {
        let content = "# My Skill\n\nThis is a test skill.";
        let hash = compute_skill_hash(content);

        // SHA256 produces 64 hex characters
        assert_eq!(hash.len(), 64);

        // Hash should be deterministic
        assert_eq!(hash, compute_skill_hash(content));
    }

    #[test]
    fn test_verify_hash() {
        let content = "# My Skill\n\nThis is a test skill.";
        let hash = compute_skill_hash(content);

        assert!(verify_skill_hash(content, &hash));
        assert!(verify_skill_hash(content, &hash.to_uppercase()));
        assert!(!verify_skill_hash("different content", &hash));
    }
}
