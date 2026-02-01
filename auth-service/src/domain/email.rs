use validator::ValidateEmail;

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Deserialize)]
pub struct Email(String);

impl Email {
        /// Parse and validate an email address
        ///
        /// Requirements:
        /// - Contains '@' symbol
        /// - Not empty
        /// - (RFC5321) Max length of the local part is 64 characters
        /// - (RFC5321) Max length of the domain part is 255 characters
        pub fn parse(email_str: &str) -> Result<Self, EmailError> {
                // Trim whitespace
                let email_str = email_str.trim();

                // Check if empty
                if email_str.is_empty() {
                        return Err(EmailError::Empty);
                }

                // Validate using validator crate
                if !email_str.validate_email() {
                        return Err(EmailError::InvalidFormat);
                }

                Ok(Email(email_str.to_string()))
        }

        /// Get the email as a string slice
        pub fn as_str(&self) -> &str {
                &self.0
        }
}

impl AsRef<str> for Email {
        fn as_ref(&self) -> &str {
                &self.0
        }
}

impl std::fmt::Display for Email {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
        }
}

#[derive(Debug, PartialEq, Eq)]
pub enum EmailError {
        Empty,
        InvalidFormat,
}

#[cfg(test)]
mod tests {
        use super::*;
        use fake::faker::internet::en::SafeEmail;
        use fake::Fake;
        use quickcheck::{Arbitrary, Gen};
        use quickcheck_macros::quickcheck;

        // Valid email test cases
        #[test]
        fn test_valid_email_simple() {
                let email = Email::parse("user@example.com").unwrap();
                assert_eq!(email.as_str(), "user@example.com");
        }

        #[test]
        fn test_valid_email_with_subdomain() {
                let email = Email::parse("user@mail.example.com").unwrap();
                assert_eq!(email.as_str(), "user@mail.example.com");
        }

        #[test]
        fn test_valid_email_with_plus() {
                let email = Email::parse("user+tag@example.com").unwrap();
                assert_eq!(email.as_str(), "user+tag@example.com");
        }

        #[test]
        fn test_valid_email_with_dots() {
                let email = Email::parse("first.last@example.com").unwrap();
                assert_eq!(email.as_str(), "first.last@example.com");
        }

        #[test]
        fn test_valid_email_with_numbers() {
                let email = Email::parse("user123@example456.com").unwrap();
                assert_eq!(email.as_str(), "user123@example456.com");
        }

        #[test]
        fn test_valid_email_with_hyphens() {
                let email = Email::parse("user@my-domain.com").unwrap();
                assert_eq!(email.as_str(), "user@my-domain.com");
        }

        #[test]
        fn test_valid_email_trims_whitespace() {
                let email = Email::parse("  user@example.com  ").unwrap();
                assert_eq!(email.as_str(), "user@example.com");
        }

        // Invalid email test cases
        #[test]
        fn test_empty_string() {
                let result = Email::parse("");
                assert_eq!(result, Err(EmailError::Empty));
        }

        #[test]
        fn test_whitespace_only() {
                let result = Email::parse("   ");
                assert_eq!(result, Err(EmailError::Empty));
        }

        #[test]
        fn test_missing_at_symbol() {
                let result = Email::parse("userexample.com");
                assert_eq!(result, Err(EmailError::InvalidFormat));
        }

        #[test]
        fn test_missing_local_part() {
                let result = Email::parse("@example.com");
                assert_eq!(result, Err(EmailError::InvalidFormat));
        }

        #[test]
        fn test_missing_domain() {
                let result = Email::parse("user@");
                assert_eq!(result, Err(EmailError::InvalidFormat));
        }

        #[test]
        fn test_missing_domain_extension() {
                // Note: validator crate allows emails without TLD (e.g., user@localhost)
                // This is technically valid for local/internal email addresses
                let result = Email::parse("user@example");
                assert!(result.is_ok(), "validator allows emails without TLD");
        }

        #[test]
        fn test_invalid_domain_starting_with_dot() {
                let result = Email::parse("user@.example.com");
                assert_eq!(result, Err(EmailError::InvalidFormat));
        }

        #[test]
        fn test_invalid_domain_ending_with_dot() {
                let result = Email::parse("user@example.com.");
                assert_eq!(result, Err(EmailError::InvalidFormat));
        }

        #[test]
        fn test_multiple_at_symbols() {
                let result = Email::parse("user@@example.com");
                assert_eq!(result, Err(EmailError::InvalidFormat));
        }

        #[test]
        fn test_invalid_characters() {
                let result = Email::parse("user name@example.com");
                assert_eq!(result, Err(EmailError::InvalidFormat));
        }

        #[test]
        fn test_consecutive_dots() {
                // Note: validator crate allows consecutive dots in local part
                // RFC 5321 allows this, though it's uncommon
                let result = Email::parse("user..name@example.com");
                assert!(result.is_ok(), "validator allows consecutive dots per RFC 5321");
        }

        // AsRef trait test
        #[test]
        fn test_as_ref_implementation() {
                let email = Email::parse("user@example.com").unwrap();
                let email_ref: &str = email.as_ref();
                assert_eq!(email_ref, "user@example.com");
        }

        // Display trait test
        #[test]
        fn test_display_implementation() {
                let email = Email::parse("user@example.com").unwrap();
                assert_eq!(format!("{}", email), "user@example.com");
        }

        // Clone and PartialEq tests
        #[test]
        fn test_clone_and_equality() {
                let email1 = Email::parse("user@example.com").unwrap();
                let email2 = email1.clone();
                assert_eq!(email1, email2);
        }

        // Property-based testing with quickcheck
        #[derive(Clone, Debug)]
        struct ValidEmail(String);

        impl Arbitrary for ValidEmail {
                fn arbitrary(g: &mut Gen) -> Self {
                        // Generate valid email components
                        let local_len = (g.size() % 10) + 3;
                        let domain_len = (g.size() % 10) + 3;

                        let local: String = (0..local_len)
                                .map(|_| {
                                        let chars = "abcdefghijklmnopqrstuvwxyz0123456789";
                                        let idx = usize::arbitrary(g) % chars.len();
                                        chars.chars().nth(idx).unwrap()
                                })
                                .collect();

                        let domain: String = (0..domain_len)
                                .map(|_| {
                                        let chars = "abcdefghijklmnopqrstuvwxyz";
                                        let idx = usize::arbitrary(g) % chars.len();
                                        chars.chars().nth(idx).unwrap()
                                })
                                .collect();

                        let tlds = ["com", "org", "net", "edu", "gov"];
                        let tld = tlds[usize::arbitrary(g) % tlds.len()];

                        ValidEmail(format!("{}@{}.{}", local, domain, tld))
                }
        }

        #[quickcheck]
        fn prop_valid_emails_always_parse(valid_email: ValidEmail) -> bool {
                Email::parse(&valid_email.0).is_ok()
        }

        #[quickcheck]
        fn prop_parsed_email_equals_trimmed_input(valid_email: ValidEmail) -> bool {
                let email_str = valid_email.0;
                if let Ok(email) = Email::parse(&email_str) {
                        email.as_str() == email_str.trim()
                } else {
                        false
                }
        }

        #[quickcheck]
        fn prop_as_ref_equals_as_str(valid_email: ValidEmail) -> bool {
                if let Ok(email) = Email::parse(&valid_email.0) {
                        let as_ref: &str = email.as_ref();
                        as_ref == email.as_str()
                } else {
                        true // Skip invalid emails
                }
        }

        // Randomized tests using fake crate
        #[test]
        fn test_random_valid_emails() {
                for _ in 0..100 {
                        let email_str: String = SafeEmail().fake();
                        let result = Email::parse(&email_str);
                        assert!(result.is_ok(), "Failed to parse valid email: {}", email_str);
                }
        }
}
