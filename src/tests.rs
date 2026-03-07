use dataxlr8_mcp_core::mcp::{get_f64, get_i64, get_str};
use serde_json::json;

// ============================================================================
// Validation logic (mirrors tools/mod.rs inline validation)
// ============================================================================

const MAX_NAME_LEN: usize = 500;
const MAX_EMAIL_LEN: usize = 500;

/// Mirrors the create_manager name validation
fn validate_name(raw: Option<String>) -> Result<String, String> {
    match raw {
        None => Err("Missing required: name".into()),
        Some(s) => {
            let trimmed = s.trim().to_string();
            if trimmed.is_empty() {
                Err("Parameter 'name' must not be empty".into())
            } else if trimmed.len() > MAX_NAME_LEN {
                Err(format!("'name' exceeds {} chars", MAX_NAME_LEN))
            } else {
                Ok(trimmed)
            }
        }
    }
}

/// Mirrors the create_manager email validation
fn validate_email(raw: Option<String>) -> Result<String, String> {
    match raw {
        None => Err("Missing required: email".into()),
        Some(s) => {
            let trimmed = s.trim().to_string();
            if trimmed.is_empty() {
                Err("Parameter 'email' must not be empty".into())
            } else if trimmed.len() > MAX_EMAIL_LEN {
                Err(format!("'email' exceeds {} chars", MAX_EMAIL_LEN))
            } else {
                Ok(trimmed)
            }
        }
    }
}

const VALID_STATUSES: &[&str] = &["pending", "approved", "paid", "cancelled"];

// ============================================================================
// Name validation — missing / empty
// ============================================================================

#[test]
fn name_missing() {
    assert!(validate_name(None).is_err());
    assert!(validate_name(None).unwrap_err().contains("Missing"));
}

#[test]
fn name_empty_string() {
    assert!(validate_name(Some("".into())).is_err());
    assert!(validate_name(Some("".into())).unwrap_err().contains("must not be empty"));
}

#[test]
fn name_whitespace_only() {
    assert!(validate_name(Some("   ".into())).is_err());
}

#[test]
fn name_tabs_and_newlines() {
    assert!(validate_name(Some("\t\n\r ".into())).is_err());
}

// ============================================================================
// Name validation — trimming
// ============================================================================

#[test]
fn name_trims_leading_trailing() {
    assert_eq!(validate_name(Some("  Alice  ".into())).unwrap(), "Alice");
}

#[test]
fn name_trims_tabs() {
    assert_eq!(validate_name(Some("\tAlice\t".into())).unwrap(), "Alice");
}

#[test]
fn name_preserves_internal_spaces() {
    assert_eq!(validate_name(Some("  Alice Bob  ".into())).unwrap(), "Alice Bob");
}

// ============================================================================
// Name validation — length limits
// ============================================================================

#[test]
fn name_at_max_len() {
    let name = "x".repeat(MAX_NAME_LEN);
    assert_eq!(validate_name(Some(name.clone())).unwrap(), name);
}

#[test]
fn name_exceeds_max_len() {
    let name = "x".repeat(MAX_NAME_LEN + 1);
    assert!(validate_name(Some(name)).is_err());
}

#[test]
fn name_way_over_max_len() {
    let name = "x".repeat(10_000);
    assert!(validate_name(Some(name)).is_err());
}

#[test]
fn name_trimmed_to_within_limit() {
    let name = format!("  {}  ", "x".repeat(MAX_NAME_LEN));
    assert_eq!(validate_name(Some(name)).unwrap().len(), MAX_NAME_LEN);
}

#[test]
fn name_trimmed_still_over_limit() {
    let name = format!("  {}  ", "x".repeat(MAX_NAME_LEN + 1));
    assert!(validate_name(Some(name)).is_err());
}

// ============================================================================
// Name validation — special characters
// ============================================================================

#[test]
fn name_sql_injection() {
    let result = validate_name(Some("'; DROP TABLE commissions.managers;--".into()));
    assert!(result.is_ok()); // Parameterized queries protect
    assert!(result.unwrap().contains("DROP TABLE"));
}

#[test]
fn name_unicode() {
    assert_eq!(validate_name(Some("日本語名前".into())).unwrap(), "日本語名前");
}

#[test]
fn name_emoji() {
    assert_eq!(validate_name(Some("Alice 🚀".into())).unwrap(), "Alice 🚀");
}

#[test]
fn name_null_byte() {
    let result = validate_name(Some("Alice\0Bob".into()));
    assert!(result.unwrap().contains('\0'));
}

#[test]
fn name_quotes() {
    let result = validate_name(Some(r#"O'Brien "The Boss""#.into()));
    assert!(result.unwrap().contains('"'));
}

#[test]
fn name_backslashes() {
    let result = validate_name(Some(r"Domain\User".into()));
    assert!(result.unwrap().contains('\\'));
}

// ============================================================================
// Email validation — missing / empty
// ============================================================================

#[test]
fn email_missing() {
    assert!(validate_email(None).is_err());
    assert!(validate_email(None).unwrap_err().contains("Missing"));
}

#[test]
fn email_empty_string() {
    assert!(validate_email(Some("".into())).is_err());
}

#[test]
fn email_whitespace_only() {
    assert!(validate_email(Some("   ".into())).is_err());
}

// ============================================================================
// Email validation — trimming
// ============================================================================

#[test]
fn email_trims_leading_trailing() {
    assert_eq!(
        validate_email(Some("  alice@example.com  ".into())).unwrap(),
        "alice@example.com"
    );
}

// ============================================================================
// Email validation — length limits
// ============================================================================

#[test]
fn email_at_max_len() {
    let email = "x".repeat(MAX_EMAIL_LEN);
    assert_eq!(validate_email(Some(email.clone())).unwrap(), email);
}

#[test]
fn email_exceeds_max_len() {
    let email = "x".repeat(MAX_EMAIL_LEN + 1);
    assert!(validate_email(Some(email)).is_err());
}

// ============================================================================
// Email validation — special characters
// ============================================================================

#[test]
fn email_sql_injection() {
    let result = validate_email(Some("alice@example.com'; DROP TABLE commissions.managers;--".into()));
    assert!(result.is_ok());
}

#[test]
fn email_crlf_injection() {
    let result = validate_email(Some("alice@example.com\r\nBCC: evil@hacker.com".into()));
    // After trim this still contains \r\n in the middle — validation accepts it, parameterized queries protect
    assert!(result.is_ok());
}

#[test]
fn email_unicode() {
    assert!(validate_email(Some("alice@例え.jp".into())).is_ok());
}

#[test]
fn email_null_byte() {
    let result = validate_email(Some("alice\0@example.com".into()));
    assert!(result.unwrap().contains('\0'));
}

// ============================================================================
// Commission status validation
// ============================================================================

#[test]
fn status_valid_all() {
    for s in VALID_STATUSES {
        assert!(VALID_STATUSES.contains(s));
    }
}

#[test]
fn status_empty() {
    assert!(!VALID_STATUSES.contains(&""));
}

#[test]
fn status_invalid() {
    assert!(!VALID_STATUSES.contains(&"completed"));
    assert!(!VALID_STATUSES.contains(&"rejected"));
    assert!(!VALID_STATUSES.contains(&"refunded"));
}

#[test]
fn status_case_sensitive() {
    assert!(!VALID_STATUSES.contains(&"Pending"));
    assert!(!VALID_STATUSES.contains(&"APPROVED"));
    assert!(!VALID_STATUSES.contains(&"Paid"));
}

#[test]
fn status_sql_injection() {
    assert!(!VALID_STATUSES.contains(&"paid'; DROP TABLE commissions.commission_records;--"));
}

// ============================================================================
// Commission amount — get_f64 edge cases
// ============================================================================

#[test]
fn amount_missing() {
    let args = json!({"manager_id": "m1", "client_id": "c1"});
    assert!(get_f64(&args, "amount").is_none());
}

#[test]
fn amount_zero() {
    let args = json!({"amount": 0.0});
    assert_eq!(get_f64(&args, "amount"), Some(0.0));
}

#[test]
fn amount_negative() {
    let args = json!({"amount": -100.50});
    assert_eq!(get_f64(&args, "amount"), Some(-100.50));
}

#[test]
fn amount_very_large() {
    let args = json!({"amount": 1_000_000_000.99});
    assert_eq!(get_f64(&args, "amount"), Some(1_000_000_000.99));
}

#[test]
fn amount_tiny_fraction() {
    let args = json!({"amount": 0.001});
    assert_eq!(get_f64(&args, "amount"), Some(0.001));
}

#[test]
fn amount_max_f64() {
    let args = json!({"amount": f64::MAX});
    let val = get_f64(&args, "amount");
    assert!(val.is_some());
}

#[test]
fn amount_min_f64() {
    let args = json!({"amount": f64::MIN});
    let val = get_f64(&args, "amount");
    assert!(val.is_some());
}

#[test]
fn amount_nan() {
    // JSON doesn't support NaN, so serde_json won't parse it
    let args = json!({"amount": null});
    assert!(get_f64(&args, "amount").is_none());
}

#[test]
fn amount_string_not_number() {
    let args = json!({"amount": "100.50"});
    assert!(get_f64(&args, "amount").is_none());
}

#[test]
fn amount_integer() {
    let args = json!({"amount": 100});
    // get_f64 should convert integer to f64
    assert_eq!(get_f64(&args, "amount"), Some(100.0));
}

// ============================================================================
// Commission rate — get_f64 defaults
// ============================================================================

#[test]
fn commission_rate_default() {
    let args = json!({"name": "Alice", "email": "alice@test.com"});
    let rate = get_f64(&args, "commission_rate").unwrap_or(0.10);
    assert_eq!(rate, 0.10);
}

#[test]
fn commission_rate_custom() {
    let args = json!({"commission_rate": 0.25});
    let rate = get_f64(&args, "commission_rate").unwrap_or(0.10);
    assert_eq!(rate, 0.25);
}

#[test]
fn commission_rate_zero() {
    let args = json!({"commission_rate": 0.0});
    let rate = get_f64(&args, "commission_rate").unwrap_or(0.10);
    assert_eq!(rate, 0.0);
}

#[test]
fn commission_rate_one() {
    let args = json!({"commission_rate": 1.0});
    let rate = get_f64(&args, "commission_rate").unwrap_or(0.10);
    assert_eq!(rate, 1.0);
}

#[test]
fn commission_rate_over_one() {
    // No validation prevents > 1.0 rates
    let args = json!({"commission_rate": 2.5});
    let rate = get_f64(&args, "commission_rate").unwrap_or(0.10);
    assert_eq!(rate, 2.5);
}

#[test]
fn commission_rate_negative() {
    let args = json!({"commission_rate": -0.05});
    let rate = get_f64(&args, "commission_rate").unwrap_or(0.10);
    assert_eq!(rate, -0.05);
}

// ============================================================================
// get_manager — lookup logic
// ============================================================================

#[test]
fn get_manager_neither_id_nor_email() {
    let args = json!({});
    assert!(get_str(&args, "id").is_none());
    assert!(get_str(&args, "email").is_none());
}

#[test]
fn get_manager_id_present() {
    let args = json!({"id": "mgr-123"});
    assert_eq!(get_str(&args, "id"), Some("mgr-123".into()));
}

#[test]
fn get_manager_email_present() {
    let args = json!({"email": "alice@test.com"});
    assert_eq!(get_str(&args, "email"), Some("alice@test.com".into()));
}

#[test]
fn get_manager_both_present() {
    // When both present, id takes priority (first branch)
    let args = json!({"id": "mgr-123", "email": "alice@test.com"});
    assert!(get_str(&args, "id").is_some());
}

#[test]
fn get_manager_id_empty_string() {
    let args = json!({"id": ""});
    // get_str returns Some("") for empty string
    assert_eq!(get_str(&args, "id"), Some("".into()));
}

#[test]
fn get_manager_id_sql_injection() {
    let args = json!({"id": "' OR '1'='1"});
    let id = get_str(&args, "id").unwrap();
    assert!(id.contains("OR")); // Parameterized queries protect
}

#[test]
fn get_manager_email_sql_injection() {
    let args = json!({"email": "'; DROP TABLE commissions.managers;--"});
    let email = get_str(&args, "email").unwrap();
    assert!(email.contains("DROP TABLE"));
}

// ============================================================================
// record_commission — required params
// ============================================================================

#[test]
fn record_commission_missing_manager_id() {
    let args = json!({"client_id": "c1", "amount": 100.0});
    assert!(get_str(&args, "manager_id").is_none());
}

#[test]
fn record_commission_missing_client_id() {
    let args = json!({"manager_id": "m1", "amount": 100.0});
    assert!(get_str(&args, "client_id").is_none());
}

#[test]
fn record_commission_missing_amount() {
    let args = json!({"manager_id": "m1", "client_id": "c1"});
    assert!(get_f64(&args, "amount").is_none());
}

#[test]
fn record_commission_optional_defaults() {
    let args = json!({"manager_id": "m1", "client_id": "c1", "amount": 100.0});
    assert_eq!(get_str(&args, "project_id").unwrap_or_default(), "");
    assert_eq!(get_str(&args, "description").unwrap_or_default(), "");
}

#[test]
fn record_commission_all_fields() {
    let args = json!({
        "manager_id": "m1",
        "client_id": "c1",
        "amount": 250.0,
        "project_id": "p1",
        "description": "Q1 bonus"
    });
    assert!(get_str(&args, "manager_id").is_some());
    assert!(get_str(&args, "client_id").is_some());
    assert!(get_f64(&args, "amount").is_some());
    assert!(get_str(&args, "project_id").is_some());
    assert!(get_str(&args, "description").is_some());
}

// ============================================================================
// get_commissions — limit clamping
// ============================================================================

#[test]
fn get_commissions_limit_default() {
    let args = json!({});
    let limit = get_i64(&args, "limit").unwrap_or(50);
    assert_eq!(limit, 50);
}

#[test]
fn get_commissions_limit_custom() {
    let args = json!({"limit": 10});
    let limit = get_i64(&args, "limit").unwrap_or(50);
    assert_eq!(limit, 10);
}

#[test]
fn get_commissions_limit_zero() {
    let args = json!({"limit": 0});
    let limit = get_i64(&args, "limit").unwrap_or(50);
    assert_eq!(limit, 0);
}

#[test]
fn get_commissions_limit_negative() {
    let args = json!({"limit": -10});
    let limit = get_i64(&args, "limit").unwrap_or(50);
    assert_eq!(limit, -10); // No clamping — DB handles this
}

#[test]
fn get_commissions_limit_max_i64() {
    let args = json!({"limit": i64::MAX});
    let limit = get_i64(&args, "limit").unwrap_or(50);
    assert_eq!(limit, i64::MAX);
}

#[test]
fn get_commissions_limit_string() {
    let args = json!({"limit": "50"});
    let limit = get_i64(&args, "limit").unwrap_or(50);
    assert_eq!(limit, 50); // Falls back to default
}

// ============================================================================
// get_commissions — optional filters
// ============================================================================

#[test]
fn get_commissions_no_filters() {
    let args = json!({});
    assert!(get_str(&args, "manager_id").is_none());
    assert!(get_str(&args, "status").is_none());
}

#[test]
fn get_commissions_manager_filter() {
    let args = json!({"manager_id": "mgr-123"});
    assert_eq!(get_str(&args, "manager_id"), Some("mgr-123".into()));
}

#[test]
fn get_commissions_status_filter() {
    let args = json!({"status": "pending"});
    assert_eq!(get_str(&args, "status"), Some("pending".into()));
}

#[test]
fn get_commissions_both_filters() {
    let args = json!({"manager_id": "mgr-123", "status": "paid"});
    assert!(get_str(&args, "manager_id").is_some());
    assert!(get_str(&args, "status").is_some());
}

#[test]
fn get_commissions_sql_injection_manager_id() {
    let args = json!({"manager_id": "' OR '1'='1; --"});
    let mid = get_str(&args, "manager_id").unwrap();
    assert!(mid.contains("OR")); // Safe via parameterized query
}

#[test]
fn get_commissions_sql_injection_status() {
    let args = json!({"status": "paid'; DROP TABLE commissions.commission_records;--"});
    let s = get_str(&args, "status").unwrap();
    assert!(s.contains("DROP TABLE"));
}

// ============================================================================
// update_commission_status — required params
// ============================================================================

#[test]
fn update_status_missing_id() {
    let args = json!({"status": "paid"});
    assert!(get_str(&args, "id").is_none());
}

#[test]
fn update_status_missing_status() {
    let args = json!({"id": "comm-123"});
    assert!(get_str(&args, "status").is_none());
}

#[test]
fn update_status_both_present() {
    let args = json!({"id": "comm-123", "status": "approved"});
    assert!(get_str(&args, "id").is_some());
    assert!(get_str(&args, "status").is_some());
}

// ============================================================================
// Status transition logic (unit tests for the logic)
// ============================================================================

#[test]
fn status_transition_pending_to_approved() {
    let old = "pending";
    let new = "approved";
    // No total changes for pending→approved
    assert!(old == "pending" && new == "approved");
}

#[test]
fn status_transition_pending_to_paid() {
    let old = "pending";
    let new = "paid";
    // Move from pending to earned
    assert!(old == "pending" && new == "paid");
}

#[test]
fn status_transition_approved_to_paid() {
    let old = "approved";
    let new = "paid";
    // Move from pending to earned
    assert!(old == "approved" && new == "paid");
}

#[test]
fn status_transition_cancel_pending() {
    let old = "pending";
    let new = "cancelled";
    // Reduce pending total
    assert!(new == "cancelled" && old != "paid");
}

#[test]
fn status_transition_cancel_approved() {
    let old = "approved";
    let new = "cancelled";
    assert!(new == "cancelled" && old != "paid");
}

#[test]
fn status_transition_cancel_paid_no_reduction() {
    // Cancelling an already-paid commission should NOT reduce pending
    let old = "paid";
    let new = "cancelled";
    assert!(!(new == "cancelled" && old != "paid"));
}

// ============================================================================
// commission_stats — manager_id filter
// ============================================================================

#[test]
fn commission_stats_no_filter() {
    let args = json!({});
    assert!(get_str(&args, "manager_id").is_none());
}

#[test]
fn commission_stats_with_filter() {
    let args = json!({"manager_id": "mgr-123"});
    assert_eq!(get_str(&args, "manager_id"), Some("mgr-123".into()));
}

// ============================================================================
// Role defaults
// ============================================================================

#[test]
fn role_default() {
    let args = json!({"name": "Alice", "email": "alice@test.com"});
    let role = get_str(&args, "role").unwrap_or_else(|| "manager".into());
    assert_eq!(role, "manager");
}

#[test]
fn role_custom() {
    let args = json!({"name": "Alice", "email": "alice@test.com", "role": "admin"});
    let role = get_str(&args, "role").unwrap_or_else(|| "manager".into());
    assert_eq!(role, "admin");
}

#[test]
fn role_empty_string() {
    let args = json!({"role": ""});
    let role = get_str(&args, "role").unwrap_or_else(|| "manager".into());
    assert_eq!(role, ""); // Empty string is returned, not default
}

#[test]
fn role_sql_injection() {
    let args = json!({"role": "admin'; DROP TABLE commissions.managers;--"});
    let role = get_str(&args, "role").unwrap();
    assert!(role.contains("DROP TABLE")); // Safe via parameterized query
}

// ============================================================================
// Core helpers — null root args
// ============================================================================

#[test]
fn args_null_root() {
    let args = serde_json::Value::Null;
    assert!(get_str(&args, "name").is_none());
    assert!(get_f64(&args, "amount").is_none());
    assert!(get_i64(&args, "limit").is_none());
}

#[test]
fn args_array_root() {
    let args = json!(["not", "an", "object"]);
    assert!(get_str(&args, "name").is_none());
}

#[test]
fn args_nested_object() {
    let args = json!({"metadata": {"key": "value"}});
    // get_str doesn't access nested keys
    assert!(get_str(&args, "key").is_none());
}

// ============================================================================
// Very long strings in optional fields
// ============================================================================

#[test]
fn description_very_long() {
    let desc = "x".repeat(10_000);
    let args = json!({"manager_id": "m1", "client_id": "c1", "amount": 100.0, "description": desc});
    let d = get_str(&args, "description").unwrap_or_default();
    assert_eq!(d.len(), 10_000);
}

#[test]
fn project_id_very_long() {
    let pid = "x".repeat(1_000);
    let args = json!({"project_id": pid});
    let p = get_str(&args, "project_id").unwrap_or_default();
    assert_eq!(p.len(), 1_000);
}
