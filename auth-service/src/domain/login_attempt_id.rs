#[derive(Debug, Clone, PartialEq)]
pub struct LoginAttemptId(String);

impl LoginAttemptId {
        pub fn parse(id: String) -> Result<Self, String> {
                // Enforce hyphenated format (must contain exactly 4 hyphens)
                if id.matches('-').count() != 4 {
                        return Err(format!(
                        "Invalid LoginAttemptID: {id}\nError: UUID must be in hyphenated format"
                    ));
                }

                let value = match uuid::Uuid::parse_str(&id) {
                        Ok(value) => value,
                        Err(e) => return Err(format!("Invalid LoginAttemptID: {id}\nError: {e}")),
                };

                Ok(LoginAttemptId(value.to_string()))
        }
}

impl Default for LoginAttemptId {
        fn default() -> Self {
                LoginAttemptId(uuid::Uuid::new_v4().to_string())
        }
}

impl AsRef<str> for LoginAttemptId {
        fn as_ref(&self) -> &str {
                &self.0
        }
}

#[cfg(test)]
mod tests {
        use super::*;

        #[test]
        fn test_parse_valid_uuids() {
                let valid_uuids = vec![
                        "550e8400-e29b-41d4-a716-446655440000", // Standard UUID v4
                        "6ba7b810-9dad-11d1-80b4-00c04fd430c8", // UUID v1
                        "6ba7b811-9dad-11d1-80b4-00c04fd430c8", // Another UUID v1
                        "00000000-0000-0000-0000-000000000000", // Nil UUID
                        "FFFFFFFF-FFFF-FFFF-FFFF-FFFFFFFFFFFF", // Max UUID (uppercase)
                        "ffffffff-ffff-ffff-ffff-ffffffffffff", // Max UUID (lowercase)
                        "123e4567-e89b-12d3-a456-426614174000", // Mixed case
                ];

                for uuid_str in valid_uuids {
                        let result = LoginAttemptId::parse(uuid_str.to_string());
                        assert!(result.is_ok(), "UUID '{}' should be valid", uuid_str);

                        let login_id = result.unwrap();
                        // UUID parsing normalizes to lowercase
                        assert_eq!(login_id.as_ref(), uuid_str.to_lowercase());
                }
        }

        #[test]
        fn test_parse_invalid_format() {
                let invalid_uuids = vec![
                        "",                                           // Empty string
                        "not-a-uuid",                                 // Random string
                        "550e8400-e29b-41d4-a716",                    // Too short
                        "550e8400-e29b-41d4-a716-446655440000-extra", // Too long
                        "550e8400-e29b-41d4-a716-44665544000",        // Missing one character
                        "550e8400-e29b-41d4-a716-4466554400000",      // Extra character
                        "550e8400e29b41d4a716446655440000",           // Missing hyphens
                        "550e8400-e29b-41d4-a716-44665544000g",       // Invalid hex character
                        "550e8400-e29b-41d4-a716-44665544000G", // Invalid hex character (uppercase)
                ];

                for uuid_str in invalid_uuids {
                        let result = LoginAttemptId::parse(uuid_str.to_string());
                        assert!(result.is_err(), "UUID '{}' should be invalid", uuid_str);

                        let error = result.unwrap_err();
                        assert!(error.starts_with("Invalid LoginAttemptID:"));
                        assert!(error.contains(uuid_str));
                        assert!(error.contains("Error:"));
                }
        }

        #[test]
        fn test_parse_invalid_characters() {
                let invalid_uuids = vec![
                        "550e8400-e29b-41d4-a716-44665544000z",   // 'z' is not hex
                        "550e8400-e29b-41d4-a716-44665544000@",   // Special character
                        "550e8400-e29b-41d4-a716-44665544000 ",   // Trailing space
                        " 550e8400-e29b-41d4-a716-446655440000",  // Leading space
                        "550e8400-e29b-41d4-a716-446655440000\n", // Newline
                        "550e8400\te29b-41d4-a716-446655440000",  // Tab instead of hyphen
                ];

                for uuid_str in invalid_uuids {
                        let result = LoginAttemptId::parse(uuid_str.to_string());
                        assert!(
                                result.is_err(),
                                "UUID '{}' should be invalid",
                                uuid_str.escape_debug()
                        );
                }
        }

        #[test]
        fn test_parse_wrong_hyphen_positions() {
                let invalid_uuids = vec![
                        "550e8400e29b-41d4-a716-446655440000",   // Missing first hyphen
                        "550e8400-e29b41d4-a716-446655440000",   // Missing second hyphen
                        "550e8400-e29b-41d4a716-446655440000",   // Missing third hyphen
                        "550e8400-e29b-41d4-a716446655440000",   // Missing fourth hyphen
                        "550e-8400-e29b-41d4-a716-446655440000", // Extra hyphen
                        "550e8400--e29b-41d4-a716-446655440000", // Double hyphen
                ];

                for uuid_str in invalid_uuids {
                        let result = LoginAttemptId::parse(uuid_str.to_string());
                        assert!(result.is_err(), "UUID '{}' should be invalid", uuid_str);
                }
        }

        #[test]
        fn test_default_generates_valid_uuid() {
                // Test that default generates valid UUIDs
                for _ in 0..100 {
                        let login_id = LoginAttemptId::default();
                        let uuid_str = login_id.as_ref();

                        // Should be parseable by our own parse function
                        let parsed = LoginAttemptId::parse(uuid_str.to_string());
                        assert!(parsed.is_ok(), "Default UUID should be parseable: '{}'", uuid_str);

                        // Should be parseable by uuid crate directly
                        let uuid_result = uuid::Uuid::parse_str(uuid_str);
                        assert!(
                                uuid_result.is_ok(),
                                "Default UUID should be valid: '{}'",
                                uuid_str
                        );

                        // Should be 36 characters (32 hex + 4 hyphens)
                        assert_eq!(uuid_str.len(), 36, "UUID should be 36 characters");

                        // Should match UUID format pattern
                        assert!(uuid_str.matches('-').count() == 4, "UUID should have 4 hyphens");
                }
        }

        #[test]
        fn test_default_generates_unique_ids() {
                // Test that default generates unique IDs
                let mut ids = std::collections::HashSet::new();

                for _ in 0..1000 {
                        let login_id = LoginAttemptId::default();
                        let uuid_str = login_id.as_ref().to_string();

                        // Should be unique (extremely unlikely to collide with UUID v4)
                        assert!(
                                ids.insert(uuid_str.clone()),
                                "Generated duplicate UUID: {}",
                                uuid_str
                        );
                }

                assert_eq!(ids.len(), 1000, "Should have generated 1000 unique UUIDs");
        }

        #[test]
        fn test_as_ref_implementation() {
                let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
                let login_id = LoginAttemptId::parse(uuid_str.to_string()).unwrap();

                assert_eq!(login_id.as_ref(), uuid_str);

                // Test that AsRef<str> works with generic functions
                fn takes_str_ref<T: AsRef<str>>(s: &T) -> &str {
                        s.as_ref()
                }

                assert_eq!(takes_str_ref(&login_id), uuid_str);
        }

        #[test]
        fn test_clone_and_partial_eq() {
                let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
                let login_id1 = LoginAttemptId::parse(uuid_str.to_string()).unwrap();
                let login_id2 = login_id1.clone();
                let login_id3 =
                        LoginAttemptId::parse("6ba7b810-9dad-11d1-80b4-00c04fd430c8".to_string())
                                .unwrap();

                // Test PartialEq
                assert_eq!(login_id1, login_id2);
                assert_ne!(login_id1, login_id3);

                // Test Clone
                assert_eq!(login_id1.as_ref(), login_id2.as_ref());
        }

        #[test]
        fn test_debug_implementation() {
                let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
                let login_id = LoginAttemptId::parse(uuid_str.to_string()).unwrap();
                let debug_str = format!("{:?}", login_id);

                assert!(debug_str.contains("LoginAttemptId"));
                assert!(debug_str.contains(uuid_str));
        }

        #[test]
        fn test_case_insensitive_parsing() {
                let lowercase = "550e8400-e29b-41d4-a716-446655440000";
                let uppercase = "550E8400-E29B-41D4-A716-446655440000";
                let mixed_case = "550e8400-E29B-41d4-A716-446655440000";

                let result1 = LoginAttemptId::parse(lowercase.to_string()).unwrap();
                let result2 = LoginAttemptId::parse(uppercase.to_string()).unwrap();
                let result3 = LoginAttemptId::parse(mixed_case.to_string()).unwrap();

                // All should parse successfully and be equal (UUID parsing normalizes case)
                assert_eq!(result1, result2);
                assert_eq!(result1, result3);

                // All should be stored in lowercase format
                assert_eq!(result1.as_ref(), lowercase);
                assert_eq!(result2.as_ref(), lowercase);
                assert_eq!(result3.as_ref(), lowercase);
        }

        #[test]
        fn test_error_message_format() {
                let invalid_id = "not-a-uuid";
                let result = LoginAttemptId::parse(invalid_id.to_string());

                assert!(result.is_err());
                let error = result.unwrap_err();

                // Check error message format
                assert!(error.starts_with("Invalid LoginAttemptID: not-a-uuid"));
                assert!(error.contains("Error:"));
        }

        #[test]
        fn test_roundtrip_conversion() {
                // Test that parse -> as_ref -> parse works correctly
                let original_uuid = "550e8400-e29b-41d4-a716-446655440000";

                let login_id1 = LoginAttemptId::parse(original_uuid.to_string()).unwrap();
                let uuid_str = login_id1.as_ref();
                let login_id2 = LoginAttemptId::parse(uuid_str.to_string()).unwrap();

                assert_eq!(login_id1, login_id2);
                assert_eq!(login_id1.as_ref(), login_id2.as_ref());
        }

        #[test]
        fn test_uuid_version_agnostic() {
                // Test that different UUID versions are accepted
                let uuid_versions = vec![
                        "6ba7b810-9dad-11d1-80b4-00c04fd430c8", // UUID v1
                        "6ba7b811-9dad-21d1-80b4-00c04fd430c8", // UUID v2
                        "6ba7b812-9dad-31d1-80b4-00c04fd430c8", // UUID v3
                        "550e8400-e29b-41d4-a716-446655440000", // UUID v4
                        "6ba7b814-9dad-51d1-80b4-00c04fd430c8", // UUID v5
                ];

                for uuid_str in uuid_versions {
                        let result = LoginAttemptId::parse(uuid_str.to_string());
                        assert!(result.is_ok(), "UUID version should be accepted: '{}'", uuid_str);
                }
        }

        #[test]
        fn test_nil_uuid() {
                let nil_uuid = "00000000-0000-0000-0000-000000000000";
                let result = LoginAttemptId::parse(nil_uuid.to_string());

                assert!(result.is_ok(), "Nil UUID should be valid");
                assert_eq!(result.unwrap().as_ref(), nil_uuid);
        }

        #[test]
        fn test_max_uuid() {
                let max_uuid = "ffffffff-ffff-ffff-ffff-ffffffffffff";
                let result = LoginAttemptId::parse(max_uuid.to_string());

                assert!(result.is_ok(), "Max UUID should be valid");
                assert_eq!(result.unwrap().as_ref(), max_uuid);
        }
}
