//! SQL identifier sanitization utilities
//!
//! Provides validation functions to prevent SQL injection when dynamic
//! identifiers (table names, schema names, column names) must be
//! interpolated into SQL strings.

use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    /// Regex pattern for valid SQL identifiers: starts with letter or underscore,
    /// followed by letters, digits, or underscores.
    static ref IDENTIFIER_REGEX: Regex = Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]*$").unwrap();

    /// Regex pattern for valid UUID strings.
    static ref UUID_REGEX: Regex = Regex::new(
        r"^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$"
    ).unwrap();
}

/// Validates that an SQL identifier contains only safe characters.
///
/// SQL identifiers (table names, schema names, column names) cannot be
/// parameterized in prepared statements, so they must be validated
/// against a whitelist pattern when dynamically constructed.
///
/// # Pattern
/// - Must start with a letter (a-z, A-Z) or underscore (_)
/// - Subsequent characters may be letters, digits (0-9), or underscores
/// - Empty strings are rejected
///
/// # Examples
///
/// ```
/// use common::sanitize_identifier;
///
/// assert!(sanitize_identifier("users").is_ok());
/// assert!(sanitize_identifier("tenant_data_2024").is_ok());
/// assert!(sanitize_identifier("_private_table").is_ok());
/// assert!(sanitize_identifier("table;DROP").is_err());
/// assert!(sanitize_identifier("1table").is_err());
/// assert!(sanitize_identifier("").is_err());
/// assert!(sanitize_identifier("has space").is_err());
/// ```
pub fn sanitize_identifier(ident: &str) -> Result<(), String> {
    if ident.is_empty() {
        return Err("SQL identifier cannot be empty".to_string());
    }
    if !IDENTIFIER_REGEX.is_match(ident) {
        return Err(format!(
            "Invalid SQL identifier '{}': must match pattern [a-zA-Z_][a-zA-Z0-9_]*",
            ident
        ));
    }
    Ok(())
}

/// Validates a tenant schema name against the whitelist.
///
/// Only `public` or names starting with `tenant_` are allowed.
pub fn sanitize_schema_name(schema: &str) -> Result<(), String> {
    if schema != "public" && !schema.starts_with("tenant_") {
        return Err(format!(
            "Invalid schema name '{}': must be 'public' or start with 'tenant_'",
            schema
        ));
    }
    sanitize_identifier(schema)
}

/// Validates a UUID string format.
pub fn sanitize_uuid(uuid_str: &str) -> Result<(), String> {
    if !UUID_REGEX.is_match(uuid_str) {
        return Err(format!(
            "Invalid UUID format '{}': expected xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
            uuid_str
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_identifiers() {
        assert!(sanitize_identifier("users").is_ok());
        assert!(sanitize_identifier("tenant_data_2024").is_ok());
        assert!(sanitize_identifier("_private").is_ok());
        assert!(sanitize_identifier("order_2025_12").is_ok());
        assert!(sanitize_identifier("a").is_ok());
    }

    #[test]
    fn test_invalid_identifiers() {
        assert!(sanitize_identifier("").is_err());
        assert!(sanitize_identifier("1table").is_err());
        assert!(sanitize_identifier("table;DROP users").is_err());
        assert!(sanitize_identifier("table name").is_err());
        assert!(sanitize_identifier("table--comment").is_err());
        assert!(sanitize_identifier("table/**/").is_err());
    }

    #[test]
    fn test_valid_schema_names() {
        assert!(sanitize_schema_name("public").is_ok());
        assert!(sanitize_schema_name("tenant_abc123").is_ok());
        assert!(sanitize_schema_name("tenant_data").is_ok());
    }

    #[test]
    fn test_invalid_schema_names() {
        assert!(sanitize_schema_name("information_schema").is_err());
        assert!(sanitize_schema_name("pg_catalog").is_err());
        assert!(sanitize_schema_name("hacked").is_err());
        assert!(sanitize_schema_name("").is_err());
    }

    #[test]
    fn test_valid_uuid() {
        assert!(sanitize_uuid("550e8400-e29b-41d4-a716-446655440000").is_ok());
    }

    #[test]
    fn test_invalid_uuid() {
        assert!(sanitize_uuid("").is_err());
        assert!(sanitize_uuid("not-a-uuid").is_err());
        assert!(sanitize_uuid("'; DROP TABLE users; --").is_err());
    }
}
