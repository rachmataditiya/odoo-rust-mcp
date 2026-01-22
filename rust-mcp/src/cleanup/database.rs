use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::odoo::types::{OdooError, OdooResult};
use crate::odoo::unified_client::OdooClient;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupOptions {
    pub remove_test_data: Option<bool>,
    pub remove_inactive_records: Option<bool>,
    pub cleanup_drafts: Option<bool>,
    pub archive_old_records: Option<bool>,
    pub optimize_database: Option<bool>,
    pub days_threshold: Option<i64>,
    pub dry_run: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupReportSummary {
    pub test_data_removed: i64,
    pub inactive_records_archived: i64,
    pub drafts_cleaned: i64,
    pub orphan_records_removed: i64,
    pub logs_cleaned: i64,
    pub attachments_cleaned: i64,
    pub cache_cleared: bool,
    pub total_records_processed: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupDetail {
    pub operation: String,
    pub model: String,
    pub records_affected: i64,
    pub details: String,
    pub status: String, // success|warning|error
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupReport {
    pub success: bool,
    pub timestamp: String,
    pub summary: CleanupReportSummary,
    pub details: Vec<CleanupDetail>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub dry_run: bool,
}

pub async fn execute_full_cleanup(
    client: &OdooClient,
    options: CleanupOptions,
) -> OdooResult<CleanupReport> {
    let dry_run = options.dry_run.unwrap_or(false);
    let days = options.days_threshold.unwrap_or(180);
    let mut report = CleanupReport {
        success: true,
        timestamp: Utc::now().to_rfc3339(),
        summary: CleanupReportSummary {
            test_data_removed: 0,
            inactive_records_archived: 0,
            drafts_cleaned: 0,
            orphan_records_removed: 0,
            logs_cleaned: 0,
            attachments_cleaned: 0,
            cache_cleared: false,
            total_records_processed: 0,
        },
        details: vec![],
        warnings: vec![],
        errors: vec![],
        dry_run,
    };

    // 1. Remove test data
    if options.remove_test_data.unwrap_or(true) {
        match remove_test_data(client, dry_run).await {
            Ok((count, details)) => {
                report.summary.test_data_removed = count;
                report.details.extend(details);
            }
            Err(e) => {
                report.success = false;
                report.errors.push(e.to_string());
            }
        }
    }

    // 2. Archive inactive records
    if options.remove_inactive_records.unwrap_or(true) {
        match archive_inactive_records(client, days, dry_run).await {
            Ok((count, details)) => {
                report.summary.inactive_records_archived = count;
                report.details.extend(details);
            }
            Err(e) => {
                report.success = false;
                report.errors.push(e.to_string());
            }
        }
    }

    // 3. Cleanup drafts
    if options.cleanup_drafts.unwrap_or(true) {
        match cleanup_draft_documents(client, dry_run).await {
            Ok((count, details)) => {
                report.summary.drafts_cleaned = count;
                report.details.extend(details);
            }
            Err(e) => {
                report.success = false;
                report.errors.push(e.to_string());
            }
        }
    }

    // 4. Remove orphans
    match remove_orphan_records(client, dry_run).await {
        Ok((count, details)) => {
            report.summary.orphan_records_removed = count;
            report.details.extend(details);
        }
        Err(e) => {
            report.success = false;
            report.errors.push(e.to_string());
        }
    }

    // 5. Clean logs
    match cleanup_activity_logs(client, days, dry_run).await {
        Ok((count, details)) => {
            report.summary.logs_cleaned = count;
            report.details.extend(details);
        }
        Err(e) => {
            report.success = false;
            report.errors.push(e.to_string());
        }
    }

    // 6. Attachments
    match cleanup_attachments(client, days, dry_run).await {
        Ok((count, details)) => {
            report.summary.attachments_cleaned = count;
            report.details.extend(details);
        }
        Err(e) => {
            report.success = false;
            report.errors.push(e.to_string());
        }
    }

    // 7. Clear caches (best effort)
    if !dry_run {
        report.summary.cache_cleared = clear_caches(client).await.unwrap_or(false);
        if !report.summary.cache_cleared {
            report.warnings.push(
                "Cache clearing failed or partially unsupported on this database.".to_string(),
            );
        }
    }

    report.summary.total_records_processed = report.summary.test_data_removed
        + report.summary.inactive_records_archived
        + report.summary.drafts_cleaned
        + report.summary.orphan_records_removed
        + report.summary.logs_cleaned
        + report.summary.attachments_cleaned;

    Ok(report)
}

async fn remove_test_data(
    client: &OdooClient,
    dry_run: bool,
) -> OdooResult<(i64, Vec<CleanupDetail>)> {
    let mut details = vec![];
    let mut total = 0i64;

    let test_data_models: Vec<(&str, Value)> = vec![
        ("res.partner", json!([["name", "like", "Test%"]])),
        ("res.partner", json!([["name", "like", "Demo%"]])),
        ("sale.order", json!([["name", "like", "%TEST%"]])),
        ("account.move", json!([["ref", "like", "%TEST%"]])),
        ("stock.move", json!([["origin", "like", "%TEST%"]])),
    ];

    for (model, domain) in test_data_models {
        let ids = client
            .search(model, Some(domain), None, None, None, None)
            .await?;
        if ids.is_empty() {
            continue;
        }
        let record_count = ids.len() as i64;
        total += record_count;

        if dry_run {
            details.push(CleanupDetail {
                operation: "remove_test_data".to_string(),
                model: model.to_string(),
                records_affected: record_count,
                details: format!("[DRY RUN] Would remove {record_count} test/demo records"),
                status: "success".to_string(),
            });
        } else {
            let ok = client.unlink(model, ids, None).await.unwrap_or(false);
            details.push(CleanupDetail {
                operation: "remove_test_data".to_string(),
                model: model.to_string(),
                records_affected: record_count,
                details: format!("Removed {record_count} test/demo records"),
                status: if ok { "success" } else { "error" }.to_string(),
            });
        }
    }

    Ok((total, details))
}

async fn archive_inactive_records(
    client: &OdooClient,
    days_threshold: i64,
    dry_run: bool,
) -> OdooResult<(i64, Vec<CleanupDetail>)> {
    let mut details = vec![];
    let mut total = 0i64;
    let threshold_date = (Utc::now() - Duration::days(days_threshold)).date_naive();
    let threshold_date_str = threshold_date.format("%Y-%m-%d").to_string();

    let archivable_models = vec![
        ("res.partner", "write_date"),
        ("sale.order", "write_date"),
        ("account.move", "write_date"),
    ];

    for (model, date_field) in archivable_models {
        let domain = json!([[date_field, "<", threshold_date_str], ["active", "=", true]]);
        let ids = client
            .search(model, Some(domain), None, None, None, None)
            .await?;
        if ids.is_empty() {
            continue;
        }
        let record_count = ids.len() as i64;
        total += record_count;
        if dry_run {
            details.push(CleanupDetail {
                operation: "archive_inactive".to_string(),
                model: model.to_string(),
                records_affected: record_count,
                details: format!("[DRY RUN] Would archive {record_count} inactive records"),
                status: "success".to_string(),
            });
        } else {
            let ok = client
                .write(model, ids.clone(), json!({ "active": false }), None)
                .await
                .unwrap_or(false);
            details.push(CleanupDetail {
                operation: "archive_inactive".to_string(),
                model: model.to_string(),
                records_affected: record_count,
                details: format!("Archived {record_count} inactive records (no activity since {threshold_date_str})"),
                status: if ok { "success" } else { "error" }.to_string(),
            });
        }
    }

    Ok((total, details))
}

async fn cleanup_draft_documents(
    client: &OdooClient,
    dry_run: bool,
) -> OdooResult<(i64, Vec<CleanupDetail>)> {
    let mut details = vec![];
    let mut total = 0i64;
    let draft_models = vec![
        ("sale.order", "state"),
        ("account.move", "state"),
        ("purchase.order", "state"),
    ];
    for (model, state_field) in draft_models {
        let domain = json!([[state_field, "=", "draft"]]);
        let ids = client
            .search(model, Some(domain), None, None, None, None)
            .await?;
        if ids.is_empty() {
            continue;
        }
        let record_count = ids.len() as i64;
        total += record_count;
        if dry_run {
            details.push(CleanupDetail {
                operation: "cleanup_drafts".to_string(),
                model: model.to_string(),
                records_affected: record_count,
                details: format!("[DRY RUN] Would delete {record_count} draft records"),
                status: "success".to_string(),
            });
        } else {
            let ok = client
                .unlink(model, ids.clone(), None)
                .await
                .unwrap_or(false);
            details.push(CleanupDetail {
                operation: "cleanup_drafts".to_string(),
                model: model.to_string(),
                records_affected: record_count,
                details: format!("Deleted {record_count} draft records"),
                status: if ok { "success" } else { "error" }.to_string(),
            });
        }
    }
    Ok((total, details))
}

async fn remove_orphan_records(
    client: &OdooClient,
    dry_run: bool,
) -> OdooResult<(i64, Vec<CleanupDetail>)> {
    let mut details = vec![];
    let mut total = 0i64;

    let orphan_pairs = vec![
        (
            "sale.order.line",
            json!([["order_id", "=", false]]),
            "orphan sale order lines",
        ),
        (
            "account.move.line",
            json!([["move_id", "=", false]]),
            "orphan invoice lines",
        ),
    ];

    for (model, domain, label) in orphan_pairs {
        let ids = client
            .search(model, Some(domain), None, None, None, None)
            .await?;
        if ids.is_empty() {
            continue;
        }
        let record_count = ids.len() as i64;
        total += record_count;
        if dry_run {
            details.push(CleanupDetail {
                operation: "remove_orphans".to_string(),
                model: model.to_string(),
                records_affected: record_count,
                details: format!("[DRY RUN] Would remove {record_count} {label}"),
                status: "success".to_string(),
            });
        } else {
            let ok = client
                .unlink(model, ids.clone(), None)
                .await
                .unwrap_or(false);
            details.push(CleanupDetail {
                operation: "remove_orphans".to_string(),
                model: model.to_string(),
                records_affected: record_count,
                details: format!("Removed {record_count} {label}"),
                status: if ok { "success" } else { "error" }.to_string(),
            });
        }
    }

    Ok((total, details))
}

async fn cleanup_activity_logs(
    client: &OdooClient,
    days_threshold: i64,
    dry_run: bool,
) -> OdooResult<(i64, Vec<CleanupDetail>)> {
    let mut details = vec![];
    let mut total = 0i64;
    let threshold_dt = Utc::now() - Duration::days(days_threshold);
    let threshold_str = threshold_dt.to_rfc3339();

    // mail.message
    let msg_ids = client
        .search(
            "mail.message",
            Some(json!([["create_date", "<", threshold_str]])),
            None,
            None,
            None,
            None,
        )
        .await?;
    if !msg_ids.is_empty() {
        let count = msg_ids.len() as i64;
        total += count;
        if dry_run {
            details.push(CleanupDetail {
                operation: "cleanup_logs".to_string(),
                model: "mail.message".to_string(),
                records_affected: count,
                details: format!("[DRY RUN] Would delete {count} old mail messages"),
                status: "success".to_string(),
            });
        } else {
            let ok = client
                .unlink("mail.message", msg_ids.clone(), None)
                .await
                .unwrap_or(false);
            details.push(CleanupDetail {
                operation: "cleanup_logs".to_string(),
                model: "mail.message".to_string(),
                records_affected: count,
                details: format!("Deleted {count} old mail messages (before {threshold_str})"),
                status: if ok { "success" } else { "error" }.to_string(),
            });
        }
    }

    // mail.activity (done)
    let act_ids = client
        .search(
            "mail.activity",
            Some(json!([
                ["create_date", "<", threshold_str],
                ["state", "=", "done"]
            ])),
            None,
            None,
            None,
            None,
        )
        .await?;
    if !act_ids.is_empty() {
        let count = act_ids.len() as i64;
        total += count;
        if dry_run {
            details.push(CleanupDetail {
                operation: "cleanup_logs".to_string(),
                model: "mail.activity".to_string(),
                records_affected: count,
                details: format!("[DRY RUN] Would delete {count} old completed activities"),
                status: "success".to_string(),
            });
        } else {
            let ok = client
                .unlink("mail.activity", act_ids.clone(), None)
                .await
                .unwrap_or(false);
            details.push(CleanupDetail {
                operation: "cleanup_logs".to_string(),
                model: "mail.activity".to_string(),
                records_affected: count,
                details: format!("Deleted {count} old completed activities"),
                status: if ok { "success" } else { "error" }.to_string(),
            });
        }
    }

    Ok((total, details))
}

async fn cleanup_attachments(
    client: &OdooClient,
    days_threshold: i64,
    dry_run: bool,
) -> OdooResult<(i64, Vec<CleanupDetail>)> {
    let mut details = vec![];
    let mut total = 0i64;

    let threshold_date = (Utc::now() - Duration::days(days_threshold)).date_naive();
    let threshold_date_str = threshold_date.format("%Y-%m-%d").to_string();
    let ids = client
        .search(
            "ir.attachment",
            Some(json!([["create_date", "<", threshold_date_str]])),
            None,
            None,
            None,
            None,
        )
        .await?;
    if ids.is_empty() {
        return Ok((0, details));
    }

    let count = ids.len() as i64;
    total += count;
    if dry_run {
        details.push(CleanupDetail {
            operation: "cleanup_attachments".to_string(),
            model: "ir.attachment".to_string(),
            records_affected: count,
            details: format!("[DRY RUN] Would delete {count} old attachments"),
            status: "success".to_string(),
        });
    } else {
        let ok = client
            .unlink("ir.attachment", ids.clone(), None)
            .await
            .unwrap_or(false);
        details.push(CleanupDetail {
            operation: "cleanup_attachments".to_string(),
            model: "ir.attachment".to_string(),
            records_affected: count,
            details: format!("Deleted {count} old attachments (before {threshold_date_str})"),
            status: if ok { "success" } else { "error" }.to_string(),
        });
    }

    Ok((total, details))
}

async fn clear_caches(client: &OdooClient) -> Result<bool, OdooError> {
    // Best effort: those methods may not exist depending on modules/edition.
    let params = serde_json::Map::new();
    let _ = client
        .call_named("ir.ui.view", "clear_caches", None, params, None)
        .await?;

    let params2 = serde_json::Map::new();
    let _ = client
        .call_named("ir.session", "clear_session_cache", None, params2, None)
        .await?;

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cleanup_options_defaults() {
        let json = "{}";
        let options: CleanupOptions = serde_json::from_str(json).unwrap();
        assert!(options.remove_test_data.is_none());
        assert!(options.dry_run.is_none());
        assert!(options.days_threshold.is_none());
    }

    #[test]
    fn test_cleanup_options_with_values() {
        let json = r#"{
            "remove_test_data": true,
            "dry_run": true,
            "days_threshold": 90
        }"#;
        let options: CleanupOptions = serde_json::from_str(json).unwrap();
        assert_eq!(options.remove_test_data, Some(true));
        assert_eq!(options.dry_run, Some(true));
        assert_eq!(options.days_threshold, Some(90));
    }

    #[test]
    fn test_cleanup_report_summary_serialization() {
        let summary = CleanupReportSummary {
            test_data_removed: 10,
            inactive_records_archived: 20,
            drafts_cleaned: 5,
            orphan_records_removed: 3,
            logs_cleaned: 100,
            attachments_cleaned: 50,
            cache_cleared: true,
            total_records_processed: 188,
        };
        let json = serde_json::to_string(&summary).unwrap();
        assert!(json.contains("testDataRemoved")); // camelCase
        assert!(json.contains("188"));
    }

    #[test]
    fn test_cleanup_detail_serialization() {
        let detail = CleanupDetail {
            operation: "test_op".to_string(),
            model: "res.partner".to_string(),
            records_affected: 42,
            details: "Test details".to_string(),
            status: "success".to_string(),
        };
        let json = serde_json::to_string(&detail).unwrap();
        assert!(json.contains("test_op"));
        assert!(json.contains("res.partner"));
        assert!(json.contains("42"));
    }

    #[test]
    fn test_cleanup_report_new() {
        let report = CleanupReport {
            success: true,
            timestamp: "2026-01-22T00:00:00Z".to_string(),
            summary: CleanupReportSummary {
                test_data_removed: 0,
                inactive_records_archived: 0,
                drafts_cleaned: 0,
                orphan_records_removed: 0,
                logs_cleaned: 0,
                attachments_cleaned: 0,
                cache_cleared: false,
                total_records_processed: 0,
            },
            details: vec![],
            warnings: vec!["warning1".to_string()],
            errors: vec![],
            dry_run: true,
        };
        
        assert!(report.success);
        assert!(report.dry_run);
        assert_eq!(report.warnings.len(), 1);
        assert!(report.errors.is_empty());
    }

    #[test]
    fn test_cleanup_options_all_flags() {
        let json = r#"{
            "remove_test_data": false,
            "remove_inactive_records": true,
            "cleanup_drafts": false,
            "archive_old_records": true,
            "optimize_database": false,
            "days_threshold": 365,
            "dry_run": false
        }"#;
        let options: CleanupOptions = serde_json::from_str(json).unwrap();
        assert_eq!(options.remove_test_data, Some(false));
        assert_eq!(options.remove_inactive_records, Some(true));
        assert_eq!(options.cleanup_drafts, Some(false));
        assert_eq!(options.archive_old_records, Some(true));
        assert_eq!(options.optimize_database, Some(false));
        assert_eq!(options.days_threshold, Some(365));
        assert_eq!(options.dry_run, Some(false));
    }
}
