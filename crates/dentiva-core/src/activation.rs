use crate::auth;

/// A fixed offline secret resists casual string searches, not binary reverse engineering.
/// Receipt persistence and device binding belong to the native Windows installation layer.
pub fn verify_activation(input: &str) -> bool {
    input.len() == 16
        && input.bytes().all(|b| b.is_ascii_digit())
        && auth::verify(input, include_str!("activation.phc").trim())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_invalid_activation_without_exposing_secret() {
        for input in ["", "0000000000000000", "not-a-number", "00000000000000000"] {
            assert!(!verify_activation(input));
        }
    }
    #[test]
    fn verifier_is_isolated_argon2id() {
        assert!(include_str!("activation.phc").starts_with("$argon2id$v=19$m=65536,t=3,p=1$"));
    }
}
