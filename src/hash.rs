//! A convenience wrapper around a byte array representing a SHA256 hash.

use std::fmt;

use sha2::{Digest, Sha256};

/// Length of a SHA-256 hash in bytes.
const SHA256_BYTES: usize = 32;

/// A SHA-256 hash of some data.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Sha256Hash([u8; SHA256_BYTES]);

impl Sha256Hash {
    /// Parse a byte slice as SHA-256 hash.
    pub fn from_bytes(hash: &[u8]) -> Option<Sha256Hash> {
        if hash.len() == SHA256_BYTES {
            let mut sha256 = Sha256Hash([0; SHA256_BYTES]);
            sha256.0.copy_from_slice(hash);
            Some(sha256)
        } else {
            None
        }
    }

    /// Parse a hex string as SHA-256 hash.
    pub fn from_hex(hash_hex: &[u8]) -> Option<Sha256Hash> {
        if hash_hex.len() == SHA256_BYTES * 2 {
            let mut bytes = [0; SHA256_BYTES];
            hex::decode_to_slice(hash_hex, &mut bytes).ok()?;
            Some(Sha256Hash(bytes))
        } else {
            None
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl fmt::Display for Sha256Hash {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let num_bytes = formatter.precision().unwrap_or(std::usize::MAX);
        for b in self.as_bytes().iter().take(num_bytes) {
            write!(formatter, "{:02x}", b)?;
        }
        Ok(())
    }
}

impl From<Sha256> for Sha256Hash {
    fn from(hasher: Sha256) -> Sha256Hash {
        Sha256Hash::from_bytes(&hasher.result()).expect("SHA 256 is broken")
    }
}

#[cfg(test)]
mod test {
    use super::Sha256Hash;
    use sha2::{Digest, Sha256};

    #[test]
    fn test_sha256hash() {
        let mut hasher = Sha256::new();
        hasher.input(b"foo");
        // check that conversion works
        let hash: Sha256Hash = hasher.into();

        // check that pretty printing works
        assert_eq!(
            format!("{}", hash),
            "2c26b46b68ffc68ff99b453c1d30413413422d706483bfa0f98a5e886266e7ae"
        );
        assert_eq!(format!("{:.8}", hash), "2c26b46b68ffc68f"); // 8 bytes
    }

    #[test]
    fn test_sha256hash_hex_roundtrip() {
        let hash: Sha256Hash = Sha256Hash::from_hex(
            b"2c26b46b68ffc68ff99b453c1d30413413422d706483bfa0f98a5e886266e7ae",
        )
        .unwrap();

        // check that pretty printing works
        assert_eq!(
            format!("{}", hash),
            "2c26b46b68ffc68ff99b453c1d30413413422d706483bfa0f98a5e886266e7ae"
        );
        assert_eq!(format!("{:.8}", hash), "2c26b46b68ffc68f"); // 8 bytes
    }
}
