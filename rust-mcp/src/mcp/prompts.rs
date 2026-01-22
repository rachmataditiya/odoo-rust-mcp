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
