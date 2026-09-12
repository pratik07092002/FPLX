use std::env;

/// Reads the JWT signing secret from the environment. Panics on startup-shaped
/// misconfiguration rather than silently signing/verifying tokens with a
/// well-known default.
pub fn secret() -> Vec<u8> {
    env::var("JWT_SECRET")
        .expect("JWT_SECRET must be set")
        .into_bytes()
}
