//! Tests for cleanup deep module
#[cfg(test)]
mod tests {
    use rust_mcp::cleanup::deep::{DeepCleanupOptions, DeepCleanupReport, DeepCleanupSummary, DeepCleanupDetail};

    #[test]
    fn test_deep_cleanup_options_creation() {
        let options = DeepCleanupOptions {
            dry_run: Some(true),
            keep_company_defaults: Some(true),
            keep_user_accounts: Some(false),
            keep_menus: Some(true),
            keep_groups: Some(false),
        };

        assert_eq!(options.dry_run, Some(true));
        assert_eq!(options.keep_company_defaults, Some(true));
        assert_eq!(options.keep_user_accounts, Some(false));
    }

    #[test]
    fn test_deep_cleanup_options_defaults() {
        let options = DeepCleanupOptions {
            dry_run: None,
            keep_company_defaults: None,
            keep_user_accounts: None,
            keep_menus: None,
            keep_groups: None,
        };

        assert!(options.dry_run.is_none());
        assert!(options.keep_company_defaults.is_none());
    }

    #[test]
    fn test_deep_cleanup_report_creation() {
        let summary = DeepCleanupSummary {
            partners_removed: 100,
            sales_orders_removed: 50,
            invoices_removed: 30,
            purchase_orders_removed: 20,
            stock_moves_removed: 15,
            documents_removed: 10,
            contacts_removed: 5,
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
            logs_and_attachments: 500,
            total_records_removed: 730,
        };

        let report = DeepCleanupReport {
            success: true,
            timestamp: "2026-01-27T00:00:00Z".to_string(),
            summary,
            details: vec![],
            warnings: vec![],
            errors: vec![],
            dry_run: true,
            default_data_retained: vec![],
        };

        assert!(report.success);
        assert_eq!(report.summary.total_records_removed, 730);
        assert!(report.dry_run);
    }

    #[test]
    fn test_deep_cleanup_detail() {
        let detail = DeepCleanupDetail {
            model: "res.partner".to_string(),
            records_removed: 100,
            details: "Removed inactive partners".to_string(),
            status: "success".to_string(),
        };

        assert_eq!(detail.model, "res.partner");
        assert_eq!(detail.records_removed, 100);
        assert_eq!(detail.status, "success");
    }
}
