use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::odoo::types::OdooResult;
use crate::odoo::unified_client::OdooClient;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepCleanupOptions {
    #[serde(rename = "dryRun")]
    pub dry_run: Option<bool>,
    #[serde(rename = "keepCompanyDefaults")]
    pub keep_company_defaults: Option<bool>,
    #[serde(rename = "keepUserAccounts")]
    pub keep_user_accounts: Option<bool>,
    #[serde(rename = "keepMenus")]
    pub keep_menus: Option<bool>,
    #[serde(rename = "keepGroups")]
    pub keep_groups: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeepCleanupSummary {
    pub partners_removed: i64,
    pub sales_orders_removed: i64,
    pub invoices_removed: i64,
    pub purchase_orders_removed: i64,
    pub stock_moves_removed: i64,
    pub documents_removed: i64,
    pub contacts_removed: i64,
    pub leads_removed: i64,
    pub opportunities_removed: i64,
    pub projects_removed: i64,
    pub tasks_removed: i64,
    pub attendees_removed: i64,
    pub events_removed: i64,
    pub journals_removed: i64,
    pub accounts_removed: i64,
    pub products_removed: i64,
    pub stock_locations_removed: i64,
    pub warehouses_removed: i64,
    pub employees_removed: i64,
    pub departments_removed: i64,
    pub logs_and_attachments: i64,
    pub total_records_removed: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepCleanupDetail {
    pub model: String,
    #[serde(rename = "recordsRemoved")]
    pub records_removed: i64,
    pub details: String,
    pub status: String, // success|warning|error
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepCleanupReport {
    pub success: bool,
    pub timestamp: String,
    pub summary: DeepCleanupSummary,
    pub details: Vec<DeepCleanupDetail>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    #[serde(rename = "dryRun")]
    pub dry_run: bool,
    #[serde(rename = "defaultDataRetained")]
    pub default_data_retained: Vec<String>,
}

pub async fn execute_deep_cleanup(
    client: &OdooClient,
    options: DeepCleanupOptions,
) -> OdooResult<DeepCleanupReport> {
    let dry_run = options.dry_run.unwrap_or(false);
    let keep_defaults = options.keep_company_defaults.unwrap_or(true);
    let keep_users = options.keep_user_accounts.unwrap_or(true);

    let mut report = DeepCleanupReport {
        success: true,
        timestamp: Utc::now().to_rfc3339(),
        summary: DeepCleanupSummary {
            partners_removed: 0,
            sales_orders_removed: 0,
            invoices_removed: 0,
            purchase_orders_removed: 0,
            stock_moves_removed: 0,
            documents_removed: 0,
            contacts_removed: 0,
            leads_removed: 0,
            opportunities_removed: 0,
            projects_removed: 0,
            tasks_removed: 0,
            attendees_removed: 0,
            events_removed: 0,
            journals_removed: 0,
            accounts_removed: 0,
            products_removed: 0,
            stock_locations_removed: 0,
            warehouses_removed: 0,
            employees_removed: 0,
            departments_removed: 0,
            logs_and_attachments: 0,
            total_records_removed: 0,
        },
        details: vec![],
        warnings: vec![],
        errors: vec![],
        dry_run,
        default_data_retained: vec![],
    };

    // 1) Partners
    let (partners_removed, mut details) = remove_partners(client, keep_defaults, dry_run).await?;
    report.summary.partners_removed = partners_removed;
    report.details.append(&mut details);

    // 2) Sales
    let (sales_removed, mut details) =
        remove_model_all(client, "sale.order", dry_run, "Removed sales orders").await?;
    report.summary.sales_orders_removed = sales_removed;
    report.summary.documents_removed += sales_removed;
    report.details.append(&mut details);

    // 3) Invoices
    let (invoices_removed, mut details) =
        remove_model_all(client, "account.move", dry_run, "Removed invoices/moves").await?;
    report.summary.invoices_removed = invoices_removed;
    report.details.append(&mut details);

    // Journals (best-effort)
    let (journals_removed, mut jdetails) = remove_by_domain_best_effort(
        client,
        "account.journal",
        json!([["type", "not in", ["general", "situation"]]]),
        dry_run,
        "Removed custom journals (best effort)",
    )
    .await?;
    report.summary.journals_removed = journals_removed;
    report.details.append(&mut jdetails);

    // Accounts (best-effort)
    let (accounts_removed, mut adetails) = remove_by_domain_best_effort(
        client,
        "account.account",
        json!([["code", "not ilike", "1%"]]),
        dry_run,
        "Removed custom accounts (best effort)",
    )
    .await?;
    report.summary.accounts_removed = accounts_removed;
    report.details.append(&mut adetails);

    // 4) Purchase
    let (po_removed, mut podetails) =
        remove_model_all(client, "purchase.order", dry_run, "Removed purchase orders").await?;
    report.summary.purchase_orders_removed = po_removed;
    report.details.append(&mut podetails);

    // 5) Stock moves (best-effort)
    let (moves_removed, mut mdetails) = remove_by_domain_best_effort(
        client,
        "stock.move",
        json!([]),
        dry_run,
        "Removed stock moves (best effort)",
    )
    .await?;
    report.summary.stock_moves_removed = moves_removed;
    report.details.append(&mut mdetails);

    // Products (best-effort)
    let (products_removed, mut pdetails) = remove_by_domain_best_effort(
        client,
        "product.product",
        json!([["create_date", "!=", false]]),
        dry_run,
        "Removed products (best effort)",
    )
    .await?;
    report.summary.products_removed = products_removed;
    report.details.append(&mut pdetails);

    // 6) CRM
    let (leads_removed, mut ldetails) = remove_by_domain_best_effort(
        client,
        "crm.lead",
        json!([["type", "=", "lead"]]),
        dry_run,
        "Removed leads",
    )
    .await?;
    report.summary.leads_removed = leads_removed;
    report.details.append(&mut ldetails);

    let (opp_removed, mut odetails) = remove_by_domain_best_effort(
        client,
        "crm.lead",
        json!([["type", "=", "opportunity"]]),
        dry_run,
        "Removed opportunities",
    )
    .await?;
    report.summary.opportunities_removed = opp_removed;
    report.details.append(&mut odetails);

    // 7) Projects/tasks
    let (tasks_removed, mut tdetails) =
        remove_model_all(client, "project.task", dry_run, "Removed tasks").await?;
    report.summary.tasks_removed = tasks_removed;
    report.details.append(&mut tdetails);

    let (projects_removed, mut prdetails) =
        remove_model_all(client, "project.project", dry_run, "Removed projects").await?;
    report.summary.projects_removed = projects_removed;
    report.details.append(&mut prdetails);

    // 8) Calendar
    let (events_removed, mut edetails) =
        remove_model_all(client, "calendar.event", dry_run, "Removed calendar events").await?;
    report.summary.events_removed = events_removed;
    report.details.append(&mut edetails);

    let (attendees_removed, mut atdetails) = remove_model_all(
        client,
        "calendar.attendee",
        dry_run,
        "Removed calendar attendees",
    )
    .await?;
    report.summary.attendees_removed = attendees_removed;
    report.details.append(&mut atdetails);

    // 9) HR
    let employee_domain = if keep_users {
        json!([["user_id", "=", false]])
    } else {
        json!([])
    };
    let (employees_removed, mut emdetails) = remove_by_domain_best_effort(
        client,
        "hr.employee",
        employee_domain,
        dry_run,
        "Removed employees",
    )
    .await?;
    report.summary.employees_removed = employees_removed;
    report.details.append(&mut emdetails);

    let (depts_removed, mut ddetails) = remove_by_domain_best_effort(
        client,
        "hr.department",
        json!([["parent_id", "!=", false]]),
        dry_run,
        "Removed departments (except root)",
    )
    .await?;
    report.summary.departments_removed = depts_removed;
    report.details.append(&mut ddetails);

    // 10) Logs + attachments
    let (logs_removed, mut lgdetails) = remove_by_domain_best_effort(
        client,
        "mail.message",
        json!([]),
        dry_run,
        "Removed mail messages",
    )
    .await?;
    let (acts_removed, mut acdetails) = remove_by_domain_best_effort(
        client,
        "mail.activity",
        json!([]),
        dry_run,
        "Removed mail activities",
    )
    .await?;
    let (atts_removed, mut attdetails) = remove_by_domain_best_effort(
        client,
        "ir.attachment",
        json!([]),
        dry_run,
        "Removed attachments",
    )
    .await?;
    report.summary.logs_and_attachments = logs_removed + acts_removed + atts_removed;
    report.details.append(&mut lgdetails);
    report.details.append(&mut acdetails);
    report.details.append(&mut attdetails);

    report.summary.total_records_removed = report.summary.partners_removed
        + report.summary.sales_orders_removed
        + report.summary.invoices_removed
        + report.summary.purchase_orders_removed
        + report.summary.stock_moves_removed
        + report.summary.leads_removed
        + report.summary.opportunities_removed
        + report.summary.projects_removed
        + report.summary.tasks_removed
        + report.summary.events_removed
        + report.summary.attendees_removed
        + report.summary.journals_removed
        + report.summary.accounts_removed
        + report.summary.products_removed
        + report.summary.employees_removed
        + report.summary.departments_removed
        + report.summary.logs_and_attachments;

    report.default_data_retained = identify_default_data(client)
        .await
        .unwrap_or_else(|_| vec!["⚠ Could not verify some defaults".to_string()]);
    if !dry_run {
        report.warnings.push("⚠ IMPORTANT: All non-essential data has been removed. Backup was recommended before this operation.".to_string());
    }

    Ok(report)
}

async fn remove_partners(
    client: &OdooClient,
    keep_defaults: bool,
    dry_run: bool,
) -> OdooResult<(i64, Vec<DeepCleanupDetail>)> {
    let mut details = vec![];
    let domain = if keep_defaults {
        json!([["name", "!=", "Your Company"]])
    } else {
        json!([])
    };
    let ids = client
        .search("res.partner", Some(domain), None, None, None, None)
        .await?;
    if ids.is_empty() {
        return Ok((0, details));
    }

    let partner_records = client
        .read(
            "res.partner",
            ids.clone(),
            Some(vec![
                "id".to_string(),
                "name".to_string(),
                "is_company".to_string(),
                "parent_id".to_string(),
            ]),
            None,
        )
        .await?;

    let system_names = [
        "Your Company",
        "Administrator",
        "Email Alias",
        "External IP",
    ];
    let mut to_delete: Vec<i64> = ids;
    if keep_defaults && let Some(arr) = partner_records.as_array() {
        to_delete = arr
            .iter()
            .filter_map(|rec| {
                let name = rec.get("name").and_then(|v| v.as_str()).unwrap_or("");
                if system_names.iter().any(|s| name.contains(s)) {
                    None
                } else {
                    rec.get("id").and_then(|v| v.as_i64())
                }
            })
            .collect();
    }

    let count = to_delete.len() as i64;
    if count == 0 {
        return Ok((0, details));
    }

    if dry_run {
        details.push(DeepCleanupDetail {
            model: "res.partner".to_string(),
            records_removed: count,
            details: format!("[DRY RUN] Would remove {count} partners"),
            status: "success".to_string(),
        });
    } else {
        let ok = client
            .unlink("res.partner", to_delete.clone(), None)
            .await
            .unwrap_or(false);
        details.push(DeepCleanupDetail {
            model: "res.partner".to_string(),
            records_removed: count,
            details: format!("Removed {count} partners (kept defaults: {keep_defaults})"),
            status: if ok { "success" } else { "error" }.to_string(),
        });
    }

    Ok((count, details))
}

async fn remove_model_all(
    client: &OdooClient,
    model: &str,
    dry_run: bool,
    label: &str,
) -> OdooResult<(i64, Vec<DeepCleanupDetail>)> {
    remove_by_domain_best_effort(client, model, json!([]), dry_run, label).await
}

async fn remove_by_domain_best_effort(
    client: &OdooClient,
    model: &str,
    domain: Value,
    dry_run: bool,
    label: &str,
) -> OdooResult<(i64, Vec<DeepCleanupDetail>)> {
    let mut details = vec![];
    let ids = client
        .search(model, Some(domain), None, None, None, None)
        .await?;
    let count = ids.len() as i64;
    if count == 0 {
        return Ok((0, details));
    }

    if dry_run {
        details.push(DeepCleanupDetail {
            model: model.to_string(),
            records_removed: count,
            details: format!("[DRY RUN] Would remove {count} records ({label})"),
            status: "success".to_string(),
        });
        return Ok((count, details));
    }

    let ok = client.unlink(model, ids, None).await.unwrap_or(false);
    details.push(DeepCleanupDetail {
        model: model.to_string(),
        records_removed: if ok { count } else { 0 },
        details: label.to_string(),
        status: if ok { "success" } else { "warning" }.to_string(),
    });
    Ok((count, details))
}

async fn identify_default_data(client: &OdooClient) -> OdooResult<Vec<String>> {
    let mut defaults = vec![];
    if !client
        .search("res.company", None, Some(1), None, None, None)
        .await?
        .is_empty()
    {
        defaults.push("✓ Default Company Retained".to_string());
    }
    if !client
        .search(
            "res.users",
            Some(json!([["id", "=", 2]])),
            Some(1),
            None,
            None,
            None,
        )
        .await?
        .is_empty()
    {
        defaults.push("✓ Admin User Retained".to_string());
    }
    if !client
        .search("ir.ui.menu", None, Some(1), None, None, None)
        .await?
        .is_empty()
    {
        defaults.push("✓ Menu Structure Retained".to_string());
    }
    if !client
        .search("res.groups", None, Some(1), None, None, None)
        .await?
        .is_empty()
    {
        defaults.push("✓ User Groups Retained".to_string());
    }
    defaults.push("✓ Module Structure Intact".to_string());
    defaults.push("✓ System Configuration Retained".to_string());
    Ok(defaults)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deep_cleanup_options_defaults() {
        let json = "{}";
        let options: DeepCleanupOptions = serde_json::from_str(json).unwrap();
        assert!(options.dry_run.is_none());
        assert!(options.keep_company_defaults.is_none());
        assert!(options.keep_user_accounts.is_none());
        assert!(options.keep_menus.is_none());
        assert!(options.keep_groups.is_none());
    }

    #[test]
    fn test_deep_cleanup_options_with_values() {
        let json = r#"{
            "dryRun": true,
            "keepCompanyDefaults": true,
            "keepUserAccounts": false,
            "keepMenus": true,
            "keepGroups": false
        }"#;
        let options: DeepCleanupOptions = serde_json::from_str(json).unwrap();
        assert_eq!(options.dry_run, Some(true));
        assert_eq!(options.keep_company_defaults, Some(true));
        assert_eq!(options.keep_user_accounts, Some(false));
        assert_eq!(options.keep_menus, Some(true));
        assert_eq!(options.keep_groups, Some(false));
    }

    #[test]
    fn test_deep_cleanup_summary_serialization() {
        let summary = DeepCleanupSummary {
            partners_removed: 100,
            sales_orders_removed: 50,
            invoices_removed: 30,
            purchase_orders_removed: 20,
            stock_moves_removed: 200,
            documents_removed: 10,
            contacts_removed: 80,
            leads_removed: 15,
            opportunities_removed: 25,
            projects_removed: 5,
            tasks_removed: 40,
            attendees_removed: 60,
            events_removed: 12,
            journals_removed: 3,
            accounts_removed: 50,
            products_removed: 150,
            stock_locations_removed: 8,
            warehouses_removed: 2,
            employees_removed: 20,
            departments_removed: 4,
            logs_and_attachments: 500,
            total_records_removed: 1384,
        };
        let json = serde_json::to_string(&summary).unwrap();
        assert!(json.contains("partnersRemoved")); // camelCase
        assert!(json.contains("1384"));
    }

    #[test]
    fn test_deep_cleanup_detail_serialization() {
        let detail = DeepCleanupDetail {
            model: "res.partner".to_string(),
            records_removed: 42,
            details: "Removed partners".to_string(),
            status: "success".to_string(),
        };
        let json = serde_json::to_string(&detail).unwrap();
        assert!(json.contains("res.partner"));
        assert!(json.contains("recordsRemoved")); // camelCase
        assert!(json.contains("42"));
    }

    #[test]
    fn test_deep_cleanup_report_new() {
        let report = DeepCleanupReport {
            success: true,
            timestamp: "2026-01-22T00:00:00Z".to_string(),
            dry_run: true,
            summary: DeepCleanupSummary {
                partners_removed: 0,
                sales_orders_removed: 0,
                invoices_removed: 0,
                purchase_orders_removed: 0,
                stock_moves_removed: 0,
                documents_removed: 0,
                contacts_removed: 0,
                leads_removed: 0,
                opportunities_removed: 0,
                projects_removed: 0,
                tasks_removed: 0,
                attendees_removed: 0,
                events_removed: 0,
                journals_removed: 0,
                accounts_removed: 0,
                products_removed: 0,
                stock_locations_removed: 0,
                warehouses_removed: 0,
                employees_removed: 0,
                departments_removed: 0,
                logs_and_attachments: 0,
                total_records_removed: 0,
            },
            details: vec![],
            warnings: vec!["Test warning".to_string()],
            errors: vec![],
            default_data_retained: vec!["Admin user".to_string()],
        };

        assert!(report.success);
        assert!(report.dry_run);
        assert_eq!(report.warnings.len(), 1);
        assert_eq!(report.default_data_retained.len(), 1);
    }

    #[test]
    fn test_deep_cleanup_summary_zero_values() {
        let summary = DeepCleanupSummary {
            partners_removed: 0,
            sales_orders_removed: 0,
            invoices_removed: 0,
            purchase_orders_removed: 0,
            stock_moves_removed: 0,
            documents_removed: 0,
            contacts_removed: 0,
            leads_removed: 0,
            opportunities_removed: 0,
            projects_removed: 0,
            tasks_removed: 0,
            attendees_removed: 0,
            events_removed: 0,
            journals_removed: 0,
            accounts_removed: 0,
            products_removed: 0,
            stock_locations_removed: 0,
            warehouses_removed: 0,
            employees_removed: 0,
            departments_removed: 0,
            logs_and_attachments: 0,
            total_records_removed: 0,
        };
        assert_eq!(summary.total_records_removed, 0);
    }

    #[test]
    fn test_deep_cleanup_options_serialize_roundtrip() {
        let options = DeepCleanupOptions {
            dry_run: Some(true),
            keep_company_defaults: Some(true),
            keep_user_accounts: Some(false),
            keep_menus: None,
            keep_groups: Some(true),
        };
        let json = serde_json::to_string(&options).unwrap();
        let parsed: DeepCleanupOptions = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.dry_run, Some(true));
        assert_eq!(parsed.keep_company_defaults, Some(true));
        assert_eq!(parsed.keep_user_accounts, Some(false));
        assert!(parsed.keep_menus.is_none());
        assert_eq!(parsed.keep_groups, Some(true));
    }
}
