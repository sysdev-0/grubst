
use pbkdf2::pbkdf2_hmac;
use rand::Rng;
use sha2::Sha512;

/// Number of PBKDF2 iterations — matches grub-mkpasswd-pbkdf2 defaults
const PBKDF2_ITERATIONS: u32 = 10_000;
/// Salt length in bytes
const SALT_LENGTH: usize = 64;
/// Key length in bytes
const KEY_LENGTH: usize = 64;

/// Generate a cryptographically secure random password of the given length
pub fn generate_random_password(length: usize) -> String {
    let charset: &[u8] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*()-_=+[]{}|;:,.<>?";
    let mut rng = rand::thread_rng();

    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..charset.len());
            charset[idx] as char
        })
        .collect()
}

/// Generate a random salt for PBKDF2
fn generate_salt() -> Vec<u8> {
    let mut rng = rand::thread_rng();
    let mut salt = vec![0u8; SALT_LENGTH];
    for byte in salt.iter_mut() {
        *byte = rng.gen();
    }
    salt
}

/// Hash a password using PBKDF2-HMAC-SHA512
/// Returns the hash in GRUB-compatible format:
/// grub.pbkdf2.sha512.ITERATIONS.SALT_HEX.HASH_HEX
pub fn hash_password_pbkdf2(password: &str) -> String {
    let salt = generate_salt();
    let mut key = vec![0u8; KEY_LENGTH];

    pbkdf2_hmac::<Sha512>(password.as_bytes(), &salt, PBKDF2_ITERATIONS, &mut key);

    format!(
        "grub.pbkdf2.sha512.{}.{}.{}",
        PBKDF2_ITERATIONS,
        hex::encode(&salt).to_uppercase(),
        hex::encode(&key).to_uppercase()
    )
}

/// Verify a password against a PBKDF2 hash string
pub fn verify_password_pbkdf2(password: &str, hash_string: &str) -> bool {
    let parts: Vec<&str> = hash_string.split('.').collect();
    if parts.len() != 6 {
        return false;
    }

    if parts[0] != "grub" || parts[1] != "pbkdf2" || parts[2] != "sha512" {
        return false;
    }

    let iter_count: u32 = match parts[3].parse() {
        Ok(v) => v,
        Err(_) => return false,
    };

    let stored_salt = match hex::decode(parts[4]) {
        Ok(v) => v,
        Err(_) => return false,
    };

    let stored_hash = match hex::decode(parts[5]) {
        Ok(v) => v,
        Err(_) => return false,
    };

    let mut computed_key = vec![0u8; stored_hash.len()];
    pbkdf2_hmac::<Sha512>(password.as_bytes(), &stored_salt, iter_count, &mut computed_key);

    use subtle::ConstantTimeEq;
    bool::from(computed_key.as_slice().ct_eq(stored_hash.as_slice()))
}

/// Generate an authentication token by combining password hash and machine fingerprint
pub fn generate_auth_token(random_password: &str, machine_fingerprint: &str) -> String {
    use sha2::Digest;

    let mut hasher = Sha512::new();
    hasher.update(random_password.as_bytes());
    hasher.update(b"::GRUBST_AUTH::");
    hasher.update(machine_fingerprint.as_bytes());
    let result = hasher.finalize();

    hex::encode(result)
}

/// Generate a SHA-512 integrity hash for file content
pub fn integrity_hash(data: &[u8]) -> String {
    use sha2::Digest;

    let mut hasher = Sha512::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_password_length() {
        let pwd = generate_random_password(32);
        assert_eq!(pwd.len(), 32);
    }

    #[test]
    fn test_random_password_uniqueness() {
        let pwd1 = generate_random_password(32);
        let pwd2 = generate_random_password(32);
        assert_ne!(pwd1, pwd2);
    }

    #[test]
    fn test_pbkdf2_hash_format() {
        let hash = hash_password_pbkdf2("testpassword");
        assert!(hash.starts_with("grub.pbkdf2.sha512."));
        assert_eq!(hash.matches('.').count(), 5);
    }

    #[test]
    fn test_auth_token_deterministic() {
        let t1 = generate_auth_token("pass", "fp");
        let t2 = generate_auth_token("pass", "fp");
        assert_eq!(t1, t2);
    }

    #[test]
    fn test_auth_token_differs_with_fingerprint() {
        let t1 = generate_auth_token("pass", "fp1");
        let t2 = generate_auth_token("pass", "fp2");
        assert_ne!(t1, t2);
    }
}
