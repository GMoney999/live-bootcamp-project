use argon2::{
        password_hash::{rand_core::OsRng, SaltString},
        Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version,
};
use std::{error::Error, str::FromStr};

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize)]
pub struct HashedPassword(String);

impl HashedPassword {
        /// Parse and hash a raw password
        pub async fn parse(s: impl Into<String>) -> Result<Self, String> {
                let s: String = s.into();

                validate_raw_password(&s)
                        .await
                        .map_err(|e| format!("Error validating password: {}", e))?;

                // Hash the password using the helper function
                let hashed = compute_password_hash(s)
                        .await
                        .map_err(|e| format!("Failed to hash password: {}", e))?;

                Ok(Self(hashed))
        }

        /// Parse an existing password hash from the database
        pub fn parse_password_hash(hash: String) -> Result<HashedPassword, String> {
                // Validate the hash format using PasswordHash::new
                PasswordHash::new(&hash)
                        .map_err(|e| format!("Invalid password hash format: {}", e))?;

                Ok(HashedPassword(hash))
        }

        /// Verify a raw password against this hashed password
        pub async fn verify_raw_password(
                &self,
                password_candidate: &str,
        ) -> Result<(), Box<dyn Error + Send + Sync>> {
                let expected_password_hash = self.0.clone();
                let password_candidate = password_candidate.to_owned();

                // Spawn blocking task to avoid blocking the async runtime
                tokio::task::spawn_blocking(move || {
                        let parsed_hash = PasswordHash::new(&expected_password_hash)?;

                        Argon2::default()
                                .verify_password(password_candidate.as_bytes(), &parsed_hash)
                                .map_err(|e| -> Box<dyn Error + Send + Sync> { Box::new(e) })
                })
                .await
                .map_err(|e| -> Box<dyn Error + Send + Sync> {
                        format!("Task join error: {}", e).into()
                })?
        }
}

/// Helper function to compute password hash
/// NOTE: Hashing is a CPU-intensive operation. To avoid blocking other async tasks, perform hashing on a separate thread pool (tokio::task::spawn_blocking)
async fn compute_password_hash(password: String) -> Result<String, Box<dyn Error + Send + Sync>> {
        tokio::task::spawn_blocking(move || {
                let salt = SaltString::generate(&mut OsRng);
                let argon2 = Argon2::default();

                let password_hash = argon2
                        .hash_password(password.as_bytes(), &salt)
                        .map_err(|e| -> Box<dyn Error + Send + Sync> { Box::new(e) })?;

                Ok(password_hash.to_string())
        })
        .await
        .map_err(|e| -> Box<dyn Error + Send + Sync> { format!("Task join error: {}", e).into() })?
}

async fn validate_raw_password(pwd: &str) -> Result<(), String> {
        // Validate password length (adjust min/max as needed)
        if pwd.is_empty() {
                return Err("Password cannot be empty".to_string());
        }
        if pwd.chars().count() < 8 {
                return Err("Password must be at least 8 characters".to_string());
        }
        if pwd.chars().count() > 128 {
                return Err("Password must not exceed 128 characters".to_string());
        }
        // Validate password contains an uppercase letter, a digit, and a special characer
        if !pwd.chars().any(|c| c.is_ascii_uppercase()) {
                return Err("Password must contain at least one uppercase letter".to_string());
        }
        if !pwd.chars().any(|c| c.is_ascii_digit()) {
                return Err("Password must contain at least one digit".to_string());
        }
        if !pwd.chars().any(|c| c.is_ascii_alphanumeric()) {
                return Err("Password must contain at least one special character".to_string());
        }
        Ok(())
}

impl std::fmt::Display for HashedPassword {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                // Never display the actual password for security
                write!(f, "[REDACTED]")
        }
}

impl PartialEq<str> for HashedPassword {
        fn eq(&self, other: &str) -> bool {
                self.0.as_str() == other
        }
}

impl AsRef<str> for HashedPassword {
        fn as_ref(&self) -> &str {
                &self.0
        }
}

#[cfg(test)]
mod tests {
        use super::HashedPassword;
        use argon2::{
                password_hash::{rand_core::OsRng, SaltString},
                Algorithm, Argon2, Params, PasswordHasher, Version,
        };
        use fake::faker::internet::en::Password as FakePassword;
        use fake::Fake;
        use quickcheck::Gen;
        use quickcheck_macros::quickcheck;
        use rand::SeedableRng;

        #[tokio::test]
        async fn empty_string_is_rejected() {
                let password = "".to_owned();
                assert!(HashedPassword::parse(password).await.is_err());
        }

        #[tokio::test]
        async fn string_less_than_8_characters_is_rejected() {
                let password = "1234567".to_owned();
                assert!(HashedPassword::parse(password).await.is_err());
        }

        #[test]
        fn can_parse_valid_argon2_hash() {
                let raw_password = "TestPassword123";
                let salt = SaltString::generate(&mut OsRng);
                let argon2 = Argon2::new(
                        Algorithm::Argon2id,
                        Version::V0x13,
                        Params::new(15000, 2, 1, None).unwrap(),
                );

                let hash_string =
                        argon2.hash_password(raw_password.as_bytes(), &salt).unwrap().to_string();

                let hash_password =
                        HashedPassword::parse_password_hash(hash_string.clone()).unwrap();

                assert_eq!(hash_password.as_ref(), hash_string.as_str());
                assert!(hash_password.as_ref().starts_with("$argon2id$v=19$"));
        }

        #[tokio::test]
        async fn can_verify_raw_password() {
                let raw_password = "TestPassword123";
                let salt = SaltString::generate(&mut OsRng);
                let argon2 = Argon2::new(
                        Algorithm::Argon2id,
                        Version::V0x13,
                        Params::new(15000, 2, 1, None).unwrap(),
                );

                let hash_string =
                        argon2.hash_password(raw_password.as_bytes(), &salt).unwrap().to_string();

                let hash_password =
                        HashedPassword::parse_password_hash(hash_string.clone()).unwrap();

                assert_eq!(hash_password.as_ref(), hash_string.as_str());
                assert!(hash_password.as_ref().starts_with("$argon2id$v=19$"));

                let result = hash_password.verify_raw_password(raw_password).await;
                assert_eq!(result.unwrap(), ());
        }

        #[derive(Debug, Clone)]
        struct ValidPasswordFixture(pub String);

        impl quickcheck::Arbitrary for ValidPasswordFixture {
                fn arbitrary(g: &mut Gen) -> Self {
                        let seed: u64 = g.size() as u64;
                        let mut rng = rand::rngs::SmallRng::seed_from_u64(seed);
                        let password = FakePassword(8..30).fake_with_rng(&mut rng);
                        Self(password)
                }
        }

        #[tokio::test]
        #[quickcheck]
        async fn valid_passwords_are_parsed_successfully(
                valid_password: ValidPasswordFixture,
        ) -> bool {
                HashedPassword::parse(valid_password.0).await.is_ok()
        }
}
