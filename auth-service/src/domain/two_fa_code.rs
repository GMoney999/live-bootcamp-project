use rand::Rng;

#[derive(Debug, Clone, PartialEq)]
pub struct TwoFACode(String);

impl TwoFACode {
        pub fn parse(code: String) -> Result<Self, String> {
                // Check if the code is exactly 6 characters (not bytes)
                if code.chars().count() != 6 {
                        return Err(format!(
                                "Code must be exactly 6 digits, got {} characters",
                                code.chars().count()
                        ));
                }

                /// Check if all characters are digits
                if !code.chars().all(|c| c.is_ascii_digit()) {
                        return Err("Code must contain only digits (0-9)".to_string());
                }

                // All validations passed
                Ok(TwoFACode(code))
        }
}

impl Default for TwoFACode {
        fn default() -> Self {
                TwoFACode(format!("{:06}", rand::rng().random_range(0..=999_999)))
        }
}

impl AsRef<str> for TwoFACode {
        fn as_ref(&self) -> &str {
                &self.0
        }
}

#[cfg(test)]
mod tests {
        use super::*;

        #[test]
        fn test_parse_valid_codes() {
                // Test various valid 6-digit codes
                let valid_codes = vec!["123456", "000000", "999999", "000123", "100000", "654321"];

                for code in valid_codes {
                        let result = TwoFACode::parse(code.to_string());
                        assert!(result.is_ok(), "Code '{}' should be valid", code);
                        assert_eq!(result.unwrap().as_ref(), code);
                }
        }

        #[test]
        fn test_parse_invalid_length() {
                let invalid_codes = vec![
                        ("", 0),
                        ("1", 1),
                        ("12", 2),
                        ("123", 3),
                        ("1234", 4),
                        ("12345", 5),
                        ("1234567", 7),
                        ("12345678", 8),
                        ("123456789", 9),
                ];

                for (code, len) in invalid_codes {
                        let result = TwoFACode::parse(code.to_string());
                        assert!(result.is_err(), "Code '{}' should be invalid", code);
                        assert_eq!(
                                result.unwrap_err(),
                                format!("Code must be exactly 6 digits, got {} characters", len)
                        );
                }
        }

        #[test]
        fn test_parse_non_digit_characters() {
                let invalid_codes = vec![
                        "12345a", "a23456", "123a56", "123 56", "123-56", "123.56", "123,56",
                        "123!56", "123@56", "123#56", "ABCDEF", "12345.", "12345-",
                ];

                for code in invalid_codes {
                        let result = TwoFACode::parse(code.to_string());
                        assert!(result.is_err(), "Code '{}' should be invalid", code);
                        assert_eq!(result.unwrap_err(), "Code must contain only digits (0-9)");
                }
        }

        #[test]
        fn test_parse_unicode_digits() {
                // Test that non-ASCII digits are rejected
                let invalid_codes = vec![
                        "12345٠",  // Arabic-Indic digit 0
                        "12345०",  // Devanagari digit 0
                        "12345０", // Fullwidth digit 0
                ];

                for code in invalid_codes {
                        let result = TwoFACode::parse(code.to_string());
                        assert!(result.is_err(), "Code '{}' should be invalid", code);
                        assert_eq!(result.unwrap_err(), "Code must contain only digits (0-9)");
                }
        }

        #[test]
        fn test_default_generates_valid_code() {
                // Test that default generates valid codes
                for _ in 0..100 {
                        let code = TwoFACode::default();
                        let code_str = code.as_ref();

                        // Should be exactly 6 characters
                        assert_eq!(code_str.len(), 6, "Default code should be 6 digits");

                        // Should contain only digits
                        assert!(
                                code_str.chars().all(|c| c.is_ascii_digit()),
                                "Default code should contain only digits: '{}'",
                                code_str
                        );

                        // Should be parseable by our own parse function
                        let parsed = TwoFACode::parse(code_str.to_string());
                        assert!(parsed.is_ok(), "Default code should be parseable: '{}'", code_str);
                }
        }

        #[test]
        fn test_default_generates_zero_padded_codes() {
                // Test that codes less than 100000 are zero-padded
                // This is probabilistic, so we run it multiple times
                let mut found_zero_padded = false;

                for _ in 0..1000 {
                        let code = TwoFACode::default();
                        let code_str = code.as_ref();

                        if code_str.starts_with('0') {
                                found_zero_padded = true;
                                // Verify it's properly zero-padded
                                let num: u32 = code_str.parse().unwrap();
                                assert!(
                                        num < 100000,
                                        "Zero-padded code should represent number < 100000"
                                );
                                break;
                        }
                }

                // This might occasionally fail due to randomness, but very unlikely
                assert!(found_zero_padded, "Should occasionally generate zero-padded codes");
        }

        #[test]
        fn test_as_ref_implementation() {
                let code = TwoFACode::parse("123456".to_string()).unwrap();
                assert_eq!(code.as_ref(), "123456");

                // Test that AsRef<str> works with generic functions
                fn takes_str_ref<T: AsRef<str>>(s: &T) -> &str {
                        s.as_ref()
                }
                assert_eq!(takes_str_ref(&code), "123456");
        }

        #[test]
        fn test_clone_and_partial_eq() {
                let code1 = TwoFACode::parse("123456".to_string()).unwrap();
                let code2 = code1.clone();
                let code3 = TwoFACode::parse("654321".to_string()).unwrap();

                // Test PartialEq
                assert_eq!(code1, code2);
                assert_ne!(code1, code3);

                // Test Clone
                assert_eq!(code1.as_ref(), code2.as_ref());
        }

        #[test]
        fn test_debug_implementation() {
                let code = TwoFACode::parse("123456".to_string()).unwrap();
                let debug_str = format!("{:?}", code);
                assert!(debug_str.contains("TwoFACode"));
                assert!(debug_str.contains("123456"));
        }

        #[test]
        fn test_edge_cases() {
                // Test boundary values
                let boundary_cases = vec![
                        "000000", // Minimum value
                        "999999", // Maximum value
                        "100000", // First non-zero-padded value
                        "099999", // Last zero-padded value
                ];

                for code in boundary_cases {
                        let result = TwoFACode::parse(code.to_string());
                        assert!(result.is_ok(), "Boundary case '{}' should be valid", code);
                        assert_eq!(result.unwrap().as_ref(), code);
                }
        }

        #[test]
        fn test_whitespace_handling() {
                let whitespace_codes = vec![
                        " 123456",  // Leading space
                        "123456 ",  // Trailing space
                        " 123456 ", // Both
                        "123 456",  // Middle space
                        "\t123456", // Tab
                        "123456\n", // Newline
                        "\r123456", // Carriage return
                ];

                for code in whitespace_codes {
                        let result = TwoFACode::parse(code.to_string());
                        assert!(
                                result.is_err(),
                                "Code with whitespace '{}' should be invalid",
                                code.escape_debug()
                        );
                }
        }
}
