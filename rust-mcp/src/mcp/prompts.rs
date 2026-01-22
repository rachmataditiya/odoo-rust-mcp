use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_json::json;

#[derive(Debug, Clone)]
pub struct PromptDef {
    pub name: &'static str,
    pub description: &'static str,
    pub content: &'static str,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prompt {
    pub name: String,
    pub description: String,
    pub content: String,
}

pub const PROMPTS: &[PromptDef] = &[
    PromptDef {
        name: "odoo_common_models",
        description: "List of commonly used Odoo models",
        content: r#"
# Common Odoo Models (v17-19)

## Sales & CRM
- sale.order - Sales Orders
- sale.order.line - Sales Order Lines
- crm.lead - CRM Leads/Opportunities
- crm.team - Sales Teams

## Accounting & Invoicing
- account.move - Invoices & Bills
- account.move.line - Invoice/Bill Lines
- account.payment - Payments
- account.journal - Journals
- account.account - Chart of Accounts

## Inventory & Manufacturing
- stock.picking - Transfers
- stock.move - Stock Moves
- stock.warehouse - Warehouses
- stock.location - Locations
- product.product - Products (variants)
- product.template - Product Templates

## Partners & Contacts
- res.partner - Contacts/Customers/Vendors
- res.company - Companies
- res.users - Users

## HR & Employees
- hr.employee - Employees
- hr.department - Departments
- hr.leave - Time Off

## Projects & Tasks
- project.project - Projects
- project.task - Tasks

## Purchase
- purchase.order - Purchase Orders
- purchase.order.line - Purchase Order Lines
"#,
    },
    PromptDef {
        name: "odoo_domain_filters",
        description: "Guide for Odoo domain filter syntax",
        content: r#"
# Odoo Domain Filter Examples

## Basic Operators
- ['name', '=', 'John'] - Exact match
- ['name', '!=', 'John'] - Not equal
- ['age', '>', 18] - Greater than
- ['age', '>=', 18] - Greater than or equal
- ['age', '<', 65] - Less than
- ['age', '<=', 65] - Less than or equal

## String Operators
- ['name', 'like', 'John'] - Contains (case-sensitive)
- ['name', 'ilike', 'john'] - Contains (case-insensitive)
- ['email', '=like', '%@example.com'] - Pattern match
- ['name', '=ilike', 'john%'] - Pattern match (case-insensitive)

## List Operators
- ['state', 'in', ['draft', 'posted']] - In list
- ['state', 'not in', ['cancel', 'draft']] - Not in list

## Logical Operators
- ['&', ['name', '=', 'John'], ['age', '>', 18]] - AND
- ['|', ['name', '=', 'John'], ['name', '=', 'Jane']] - OR
- ['!', ['state', '=', 'cancel']] - NOT

## Complex Example
[
  '&',
  ['state', '=', 'sale'],
  '|',
  ['amount_total', '>', 1000],
  ['partner_id.country_id.code', '=', 'US']
]
"#,
    },
];

pub fn default_prompts() -> Vec<Prompt> {
    PROMPTS
        .iter()
        .map(|p| Prompt {
            name: p.name.to_string(),
            description: p.description.to_string(),
            content: p.content.to_string(),
        })
        .collect()
}

pub fn list_prompts_result(prompts: &[(String, String)]) -> Value {
    json!({
        "prompts": prompts.iter().map(|(name, description)| json!({
            "name": name,
            "description": description,
        })).collect::<Vec<_>>()
    })
}

pub fn get_prompt_result(prompt: &Prompt) -> Value {
    json!({
        "description": prompt.description,
        "messages": [
            {
                "role": "user",
                "content": {
                    "type": "text",
                    "text": prompt.content
                }
            }
        ]
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_prompts_not_empty() {
        let prompts = default_prompts();
        assert!(!prompts.is_empty());
        assert!(prompts.len() >= 2); // We have at least 2 prompts defined
    }

    #[test]
    fn test_default_prompts_have_content() {
        let prompts = default_prompts();
        for p in prompts {
            assert!(!p.name.is_empty());
            assert!(!p.description.is_empty());
            assert!(!p.content.is_empty());
        }
    }

    #[test]
    fn test_list_prompts_result_format() {
        let prompts = vec![
            ("test_prompt".to_string(), "A test prompt".to_string()),
            ("another".to_string(), "Another prompt".to_string()),
        ];
        let result = list_prompts_result(&prompts);
        
        assert!(result.is_object());
        let prompts_arr = result["prompts"].as_array().unwrap();
        assert_eq!(prompts_arr.len(), 2);
        assert_eq!(prompts_arr[0]["name"], "test_prompt");
        assert_eq!(prompts_arr[0]["description"], "A test prompt");
    }

    #[test]
    fn test_get_prompt_result_format() {
        let prompt = Prompt {
            name: "test".to_string(),
            description: "Test description".to_string(),
            content: "Test content here".to_string(),
        };
        let result = get_prompt_result(&prompt);
        
        assert!(result.is_object());
        assert_eq!(result["description"], "Test description");
        
        let messages = result["messages"].as_array().unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0]["role"], "user");
        assert_eq!(messages[0]["content"]["type"], "text");
        assert_eq!(messages[0]["content"]["text"], "Test content here");
    }

    #[test]
    fn test_prompt_serialization() {
        let prompt = Prompt {
            name: "test".to_string(),
            description: "desc".to_string(),
            content: "content".to_string(),
        };
        let json = serde_json::to_string(&prompt).unwrap();
        let parsed: Prompt = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "test");
        assert_eq!(parsed.description, "desc");
        assert_eq!(parsed.content, "content");
    }

    #[test]
    fn test_prompts_constant_has_odoo_models() {
        let found = PROMPTS.iter().any(|p| p.name == "odoo_common_models");
        assert!(found);
    }

    #[test]
    fn test_prompts_constant_has_domain_filters() {
        let found = PROMPTS.iter().any(|p| p.name == "odoo_domain_filters");
        assert!(found);
    }
}
