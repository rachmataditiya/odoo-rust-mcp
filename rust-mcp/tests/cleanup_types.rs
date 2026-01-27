//! Tests for cleanup module types
#[cfg(test)]
mod tests {
    use chrono::Utc;
    use rust_mcp::cleanup::deep::{
        DeepCleanupDetail, DeepCleanupOptions, DeepCleanupReport, DeepCleanupSummary,
    };

    #[test]
    fn test_deep_cleanup_options_defaults() {
        let options = DeepCleanupOptions {
            dry_run: Some(true),
            keep_company_defaults: Some(true),
            keep_user_accounts: Some(true),
            keep_menus: Some(false),
            keep_groups: Some(false),
        };

        assert_eq!(options.dry_run, Some(true));
        assert_eq!(options.keep_company_defaults, Some(true));
        assert_eq!(options.keep_user_accounts, Some(true));
        assert_eq!(options.keep_menus, Some(false));
        assert_eq!(options.keep_groups, Some(false));
    }

    #[test]
    fn test_deep_cleanup_options_all_none() {
        let options = DeepCleanupOptions {
            dry_run: None,
            keep_company_defaults: None,
            keep_user_accounts: None,
            keep_menus: None,
            keep_groups: None,
        };

        assert_eq!(options.dry_run, None);
        assert_eq!(options.keep_company_defaults, None);
    }

    #[test]
    fn test_deep_cleanup_summary_creation() {
        let summary = DeepCleanupSummary {
            partners_removed: 10,
            sales_orders_removed: 5,
            invoices_removed: 15,
            purchase_orders_removed: 3,
            stock_moves_removed: 20,
            documents_removed: 0,
            contacts_removed: 8,
            leads_removed: 12,
            opportunities_removed: 4,
            projects_removed: 2,
            tasks_removed: 18,
            attendees_removed: 25,
            events_removed: 6,
            journals_removed: 1,
            accounts_removed: 0,
            products_removed: 0,
            stock_locations_removed: 0,
            warehouses_removed: 0,
            employees_removed: 0,
            departments_removed: 0,
            logs_and_attachments: 50,
            total_records_removed: 179,
        };

        assert_eq!(summary.partners_removed, 10);
        assert_eq!(summary.sales_orders_removed, 5);
        assert_eq!(summary.total_records_removed, 179);
    }

    #[test]
    fn test_deep_cleanup_detail_success() {
        let detail = DeepCleanupDetail {
            model: "res.partner".to_string(),
            records_removed: 15,
            details: "Removed 15 partners".to_string(),
            status: "success".to_string(),
        };

        assert_eq!(detail.model, "res.partner");
        assert_eq!(detail.records_removed, 15);
        assert_eq!(detail.status, "success");
    }

    #[test]
    fn test_deep_cleanup_detail_warning() {
        let detail = DeepCleanupDetail {
            model: "account.account".to_string(),
            records_removed: 0,
            details: "Could not remove accounts with active balance".to_string(),
            status: "warning".to_string(),
        };

        assert_eq!(detail.status, "warning");
        assert_eq!(detail.records_removed, 0);
    }

    #[test]
    fn test_deep_cleanup_detail_error() {
        let detail = DeepCleanupDetail {
            model: "res.users".to_string(),
            records_removed: 0,
            details: "Cannot remove admin user".to_string(),
            status: "error".to_string(),
        };

        assert_eq!(detail.status, "error");
    }

    #[test]
    fn test_deep_cleanup_report_creation() {
        let summary = DeepCleanupSummary {
            partners_removed: 5,
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
            total_records_removed: 5,
        };

        let report = DeepCleanupReport {
            success: true,
            timestamp: Utc::now().to_rfc3339(),
            summary,
            details: vec![],
            warnings: vec![],
            errors: vec![],
            dry_run: false,
            default_data_retained: vec![],
        };

        assert!(report.success);
        assert!(!report.dry_run);
        assert!(report.details.is_empty());
    }

    #[test]
    fn test_deep_cleanup_report_with_warnings() {
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

        let report = DeepCleanupReport {
            success: false,
            timestamp: Utc::now().to_rfc3339(),
            summary,
            details: vec![],
            warnings: vec!["Some records could not be removed".to_string()],
            errors: vec!["Critical error occurred".to_string()],
            dry_run: true,
            default_data_retained: vec!["company_1".to_string()],
        };

        assert!(!report.success);
        assert!(report.dry_run);
        assert_eq!(report.warnings.len(), 1);
        assert_eq!(report.errors.len(), 1);
        assert_eq!(report.default_data_retained.len(), 1);
    }

    #[test]
    fn test_deep_cleanup_report_dry_run() {
        let summary = DeepCleanupSummary {
            partners_removed: 100,
            sales_orders_removed: 50,
            invoices_removed: 75,
            purchase_orders_removed: 25,
            stock_moves_removed: 150,
            documents_removed: 0,
            contacts_removed: 30,
            leads_removed: 40,
            opportunities_removed: 20,
            projects_removed: 5,
            tasks_removed: 60,
            attendees_removed: 80,
            events_removed: 15,
            journals_removed: 2,
            accounts_removed: 0,
            products_removed: 0,
            stock_locations_removed: 0,
            warehouses_removed: 0,
            employees_removed: 0,
            departments_removed: 0,
            logs_and_attachments: 200,
            total_records_removed: 752,
        };

        let report = DeepCleanupReport {
            success: true,
            timestamp: Utc::now().to_rfc3339(),
            summary,
            details: vec![],
            warnings: vec![],
            errors: vec![],
            dry_run: true,
            default_data_retained: vec!["company_1".to_string(), "default_warehouse".to_string()],
        };

        assert!(report.dry_run);
        assert_eq!(report.summary.total_records_removed, 752);
        assert_eq!(report.default_data_retained.len(), 2);
    }
}
