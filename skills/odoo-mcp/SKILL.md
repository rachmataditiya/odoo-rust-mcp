---
name: odoo-mcp
description: >-
  Autonomous Odoo ERP operations, schema inspection, workflow actions, reporting, and database queries
  via the Odoo MCP Server. Use whenever the user asks to query Odoo models (sale.order, account.move,
  stock.picking, purchase.order, res.partner, hr.employee, etc.), inspect fields and relations, perform
  safe data creation/updates, execute workflow actions, download PDF reports, check instance health,
  or interact with Odoo 14-19+ via MCP tools.
---

# Odoo MCP Expert Operations Guide

This skill guides the agent in using the **Odoo Rust MCP Server** (`rust-mcp`) effectively, safely,
and idiomatic to Odoo ERP standards across all supported versions (Odoo 14 to 19+).

---

## 1. Safety Protocols & Multi-Instance Hygiene

The MCP server connects to multiple Odoo instances configured in `~/.config/odoo-rust-mcp/instances.json`
(e.g., `production`, `staging`, `dev`).

### Safety Rules:
1. **Never perform blind mutations on `production`**:
   - If the target instance is `production` or unstated, **confirm with the user** before calling `odoo_create`, `odoo_update`, `odoo_delete`, or `odoo_execute` (mutating actions).
   - If the user simply says "check data" or "why did order X fail", query `staging` or `dev` first, or inspect `production` with **read-only** tools (`odoo_search_read`, `odoo_read`).
2. **Respect Read-Only Safety Guards**:
   - Instances configured with `"read_only": true` will strictly reject any create, write, unlink, execute, or cleanup calls at the MCP level.
3. **Always Check Health First**:
   - When encountering connectivity issues or starting a complex workflow, verify instance health via `http://127.0.0.1:8787/health`.

---

## 2. Model Schema Discovery Ritual

Odoo models vary drastically across versions and installed custom addons.
**Never guess field names.** Always discover metadata before querying or writing.

### Discovery Pattern:
```python
# 1. Inspect model fields, types, and requirements
mcp__odoo__odoo_get_model_metadata(
    instance="staging",
    model="sale.order"
)

# 2. Or query field definitions directly
mcp__odoo__odoo_execute(
    instance="staging",
    model="sale.order",
    method="fields_get",
    args=[],
    kwargs={"attributes": ["string", "type", "required", "relation"]}
)
```

Common field types to handle:
- **`many2one`**: Integer ID when writing (e.g. `{"partner_id": 42}`). Returned as `[id, name]` tuple when reading.
- **`one2many` / `many2many`**: Command tuples when writing:
  - `(0, 0, {values})`: Create new linked record
  - `(1, id, {values})`: Update existing linked record
  - `(2, id, 0)`: Delete and unlink
  - `(3, id, 0)`: Unlink without deleting
  - `(4, id, 0)`: Link existing record
  - `(6, 0, [ids])`: Replace all existing links with the list of IDs

---

## 3. Efficient Querying Best Practices

### A. Explicit Field Selection (Avoid Context Bloat)
**Do NOT** call `odoo_search_read` with empty `fields: []` unless you truly need all 100+ columns of a model.
Always select only the needed fields:
```python
mcp__odoo__odoo_search_read(
    instance="staging",
    model="account.move",
    domain=[
        ("move_type", "=", "out_invoice"),
        ("state", "=", "posted"),
        ("invoice_date", ">=", "2026-01-01")
    ],
    fields=["name", "partner_id", "amount_total", "payment_state", "invoice_date"],
    limit=20,
    order="invoice_date desc"
)
```

### B. Odoo Domain Filter Syntax (Polish Prefix Notation)
Odoo domains are lists of criteria in Polish prefix notation:
- Criterion format: `("field_name", "operator", value)`
- Available operators: `=`, `!=`, `>`, `>=`, `<`, `<=`, `like`, `ilike`, `=ilike`, `in`, `not in`, `child_of`
- Logical operators prefix criteria:
  - Default is implicit **AND** (`&`):
    `[("state", "=", "sale"), ("date_order", ">=", "2026-01-01")]`
  - Explicit **OR** (`|`) applies to the next two criteria:
    `["|", ("state", "=", "draft"), ("state", "=", "sent")]`
  - Explicit **NOT** (`!`):
    `["!", ("partner_id", "=", False)]`
  - Nested expressions:
    `["&", ("is_company", "=", True), "|", ("customer_rank", ">", 0), ("supplier_rank", ">", 0)]`

---

## 4. Standard ERP Workflows & Model Map

| Business Area | Model | Common States | Workflow Methods (`odoo_workflow_action` or `odoo_execute`) |
| :--- | :--- | :--- | :--- |
| **Sales** | `sale.order` | `draft`, `sent`, `sale`, `cancel` | `action_confirm`, `action_cancel`, `action_draft` |
| **Sales Lines**| `sale.order.line` | - | Linked via `order_line` |
| **Invoicing** | `account.move` | `draft`, `posted`, `cancel` | `action_post`, `button_cancel`, `button_draft` |
| **Payments** | `account.payment` | `draft`, `posted`, `cancel` | `action_post`, `action_cancel` |
| **Stock / WMS**| `stock.picking` | `draft`, `waiting`, `confirmed`, `assigned`, `done`, `cancel` | `action_confirm`, `action_assign`, `button_validate` |
| **Purchasing** | `purchase.order`| `draft`, `sent`, `to approve`, `purchase`, `done`, `cancel` | `button_confirm`, `button_cancel`, `button_draft` |
| **Partners** | `res.partner` | - | `is_company`, `parent_id`, `email`, `vat` |
| **Products** | `product.template` / `product.product` | - | `default_code`, `list_price`, `standard_price` |

### Triggering a Workflow Action:
```python
mcp__odoo__odoo_workflow_action(
    instance="staging",
    model="sale.order",
    id=105,
    action="action_confirm"
)
```

---

## 5. Tool Quick Reference

| MCP Tool | Purpose | Key Parameters |
| :--- | :--- | :--- |
| `odoo_search_read` | Search and retrieve records in one call | `model`, `domain`, `fields`, `limit`, `offset`, `order` |
| `odoo_read` | Retrieve specific records by ID list | `model`, `ids`, `fields` |
| `odoo_count` | Count matching records without fetching data | `model`, `domain` |
| `odoo_create` | Create a single record | `model`, `values` |
| `odoo_create_batch` | Create multiple records efficiently | `model`, `records` |
| `odoo_update` | Write updates to existing record(s) | `model`, `ids`, `values` |
| `odoo_delete` | Unlink record(s) (use with extreme care) | `model`, `ids` |
| `odoo_execute` | Call any public ORM method on a model | `model`, `method`, `args`, `kwargs` |
| `odoo_workflow_action`| Execute state transition method on a record | `model`, `id`, `action` |
| `odoo_generate_report`| Generate and download a PDF report | `report_name`, `record_ids` |
| `odoo_read_group` | Aggregated summaries (sum, count, avg) | `model`, `domain`, `fields`, `groupby` |
| `odoo_get_model_metadata`| Inspect field dictionary and schema | `model` |
| `odoo_list_models` | Search available models in the Odoo system | `filter` |
| `odoo_check_access` | Verify user CRUD permissions on a model | `model`, `operation` |
