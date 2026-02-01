#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Password(String);

impl Password {
        /// Parse and validate a password
        ///
        /// Requirements:
        /// - At least 8 characters
        /// - Not empty
        /// - Contains at least one uppercase letter
        /// - Contains at least one lowercase letter
        /// - Contains at least one digit
        pub fn parse(password: &str) -> Result<Self, PasswordError> {
                // Check if empty
                if password.is_empty() {
                        return Err(PasswordError::Empty);
                }

                // Check minimum length
                if password.chars().count() < 8 {
                        return Err(PasswordError::TooShort);
                }

                // Check maximum length to prevent DoS
                if password.chars().count() > 128 {
                        return Err(PasswordError::TooLong);
                }

                // Check for at least one uppercase letter
                if !password.chars().any(|c| c.is_uppercase()) {
                        return Err(PasswordError::MissingUppercase);
                }

                // Check for at least one lowercase letter
                if !password.chars().any(|c| c.is_lowercase()) {
                        return Err(PasswordError::MissingLowercase);
                }

                // Check for at least one digit
                if !password.chars().any(|c| c.is_ascii_digit()) {
                        return Err(PasswordError::MissingDigit);
                }

                Ok(Password(password.to_string()))
        }

        /// Get the password as a string slice
        pub fn as_str(&self) -> &str {
                &self.0
        }
}

impl std::fmt::Display for Password {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                // Never display the actual password for security
                write!(f, "[REDACTED]")
        }
}

impl AsRef<str> for Password {
        fn as_ref(&self) -> &str {
                &self.0
        }
}

impl PartialEq<str> for Password {
        fn eq(&self, other: &str) -> bool {
                self.0.as_str() == other
        }
}

#[derive(Debug, PartialEq, Eq)]
pub enum PasswordError {
        Empty,
        TooShort,
        TooLong,
        MissingUppercase,
        MissingLowercase,
        MissingDigit,
}

#[cfg(test)]
mod tests {
        use super::*;
        use quickcheck_macros::quickcheck;

        // Valid password test cases
        #[test]
        fn test_valid_password_simple() {
                let password = Password::parse("Password123").unwrap();
                assert_eq!(password.as_str(), "Password123");
        }

        #[test]
        fn test_valid_password_with_special_chars() {
                let password = Password::parse("P@ssw0rd!").unwrap();
                assert_eq!(password.as_str(), "P@ssw0rd!");
        }

        #[test]
        fn test_valid_password_exactly_8_chars() {
                let password = Password::parse("Passw0rd").unwrap();
                assert_eq!(password.as_str(), "Passw0rd");
        }

        #[test]
        fn test_valid_password_very_long() {
                let long_password = "A1".to_string() + &"a".repeat(126); // 128 chars total
                let password = Password::parse(&long_password).unwrap();
                assert_eq!(password.as_str(), &long_password);
        }

        #[test]
        fn test_valid_password_with_unicode() {
                let password = Password::parse("Pässw0rd").unwrap();
                assert_eq!(password.as_str(), "Pässw0rd");
        }

        // Empty password tests
        #[test]
        fn test_empty_password() {
                let result = Password::parse("");
                assert_eq!(result, Err(PasswordError::Empty));
        }

        // Length validation tests
        #[test]
        fn test_password_too_short_7_chars() {
                let result = Password::parse("Pass1");
                assert_eq!(result, Err(PasswordError::TooShort));
        }

        #[test]
        fn test_password_too_short_1_char() {
                let result = Password::parse("A");
                assert_eq!(result, Err(PasswordError::TooShort));
        }

        #[test]
        fn test_password_too_long() {
                let long_password = "A1".to_string() + &"a".repeat(127); // 129 chars
                let result = Password::parse(&long_password);
                assert_eq!(result, Err(PasswordError::TooLong));
        }

        // Missing character type tests
        #[test]
        fn test_password_missing_uppercase() {
                let result = Password::parse("password123");
                assert_eq!(result, Err(PasswordError::MissingUppercase));
        }

        #[test]
        fn test_password_missing_lowercase() {
                let result = Password::parse("PASSWORD123");
                assert_eq!(result, Err(PasswordError::MissingLowercase));
        }

        #[test]
        fn test_password_missing_digit() {
                let result = Password::parse("PasswordABC");
                assert_eq!(result, Err(PasswordError::MissingDigit));
        }

        #[test]
        fn test_password_missing_uppercase_and_digit() {
                let result = Password::parse("password");
                // Should fail on first check (uppercase)
                assert_eq!(result, Err(PasswordError::MissingUppercase));
        }

        #[test]
        fn test_password_only_special_chars() {
                let result = Password::parse("!@#$%^&*()");
                // Fails because no uppercase
                assert_eq!(result, Err(PasswordError::MissingUppercase));
        }

        // AsRef trait test
        #[test]
        fn test_as_ref_implementation() {
                let password = Password::parse("Password123").unwrap();
                let password_ref: &str = password.as_ref();
                assert_eq!(password_ref, "Password123");
        }

        // Display trait test - security feature
        #[test]
        fn test_display_redacts_password() {
                let password = Password::parse("Password123").unwrap();
                assert_eq!(format!("{}", password), "[REDACTED]");
        }

        // Clone and PartialEq tests
        #[test]
        fn test_clone_and_equality() {
                let password1 = Password::parse("Password123").unwrap();
                let password2 = password1.clone();
                assert_eq!(password1, password2);
        }

        #[test]
        fn test_inequality() {
                let password1 = Password::parse("Password123").unwrap();
                let password2 = Password::parse("DifferentPass1").unwrap();
                assert_ne!(password1, password2);
        }

        // Edge cases
        #[test]
        fn test_password_with_whitespace() {
                let password = Password::parse("Pass word 123").unwrap();
                assert_eq!(password.as_str(), "Pass word 123");
        }

        #[test]
        fn test_password_with_leading_trailing_spaces() {
                // Note: We don't trim passwords - spaces are valid
                let password = Password::parse(" Password123 ").unwrap();
                assert_eq!(password.as_str(), " Password123 ");
        }

        #[test]
        fn test_password_all_uppercase_letters_same() {
                let result = Password::parse("AAAAAAAA");
                assert_eq!(result, Err(PasswordError::MissingLowercase));
        }

        #[test]
        fn test_password_unicode_length_counting() {
                // Unicode characters count as single chars
                let password = Password::parse("Pä1");
                assert_eq!(password, Err(PasswordError::TooShort));
        }

        // Property-based tests
        #[quickcheck]
        fn prop_valid_password_always_parses(s: String) -> bool {
                if s.len() < 8 || s.len() > 128 {
                        return true; // Skip invalid lengths
                }
                let has_upper = s.chars().any(|c| c.is_uppercase());
                let has_lower = s.chars().any(|c| c.is_lowercase());
                let has_digit = s.chars().any(|c| c.is_ascii_digit());

                if has_upper && has_lower && has_digit {
                        Password::parse(&s).is_ok()
                } else {
                        Password::parse(&s).is_err()
                }
        }

        #[quickcheck]
        fn prop_parsed_password_equals_input(s: String) -> bool {
                if let Ok(password) = Password::parse(&s) {
                        password.as_str() == s
                } else {
                        true // Skip invalid passwords
                }
        }

        #[quickcheck]
        fn prop_display_never_reveals_password(s: String) -> bool {
                if let Ok(password) = Password::parse(&s) {
                        format!("{}", password) == "[REDACTED]"
                } else {
                        true
                }
        }

        #[quickcheck]
        fn prop_as_ref_equals_as_str(s: String) -> bool {
                if let Ok(password) = Password::parse(&s) {
                        let as_ref: &str = password.as_ref();
                        as_ref == password.as_str()
                } else {
                        true
                }
        }

        // Security-focused tests
        #[test]
        fn test_common_weak_passwords_rejected() {
                let weak_passwords = ["password", "12345678", "QWERTYUIOP", "letmein", "welcome"];

                for weak in &weak_passwords {
                        let result = Password::parse(weak);
                        assert!(result.is_err(), "Weak password '{}' should be rejected", weak);
                }
        }
}
