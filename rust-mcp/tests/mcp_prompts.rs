//! Tests for MCP Prompts module
#[cfg(test)]
mod tests {
    use rust_mcp::mcp::prompts::{PROMPTS, Prompt, PromptDef};

    #[test]
    fn test_prompt_def_creation() {
        let prompt = PromptDef {
            name: "odoo_intro",
            description: "Introduction to Odoo",
            content: "Odoo is an open-source business application suite",
        };

        assert_eq!(prompt.name, "odoo_intro");
        assert_eq!(prompt.description, "Introduction to Odoo");
        assert!(prompt.content.contains("Odoo"));
    }

    #[test]
    fn test_prompt_struct() {
        let prompt = Prompt {
            name: "test_prompt".to_string(),
            description: "Test prompt description".to_string(),
            content: "Test content for prompt".to_string(),
        };

        assert_eq!(prompt.name, "test_prompt");
        assert_eq!(prompt.description, "Test prompt description");
        assert_eq!(prompt.content, "Test content for prompt");
    }

    #[test]
    fn test_prompts_constant_not_empty() {
        assert!(!PROMPTS.is_empty());
    }

    #[test]
    fn test_prompts_have_required_fields() {
        for prompt in PROMPTS.iter() {
            assert!(!prompt.name.is_empty());
            assert!(!prompt.description.is_empty());
            assert!(!prompt.content.is_empty());
        }
    }

    #[test]
    fn test_specific_prompts_exist() {
        let prompt_names: Vec<_> = PROMPTS.iter().map(|p| p.name).collect();

        // Check for expected prompts
        assert!(
            prompt_names.contains(&"odoo_common_models"),
            "odoo_common_models prompt should exist"
        );
        assert!(
            prompt_names.contains(&"odoo_domain_filters"),
            "odoo_domain_filters prompt should exist"
        );
    }

    #[test]
    fn test_prompt_to_struct_conversion() {
        let def = PromptDef {
            name: "test",
            description: "desc",
            content: "content",
        };

        let prompt = Prompt {
            name: def.name.to_string(),
            description: def.description.to_string(),
            content: def.content.to_string(),
        };

        assert_eq!(prompt.name, def.name);
        assert_eq!(prompt.description, def.description);
        assert_eq!(prompt.content, def.content);
    }

    #[test]
    fn test_prompts_have_unique_names() {
        let names: Vec<_> = PROMPTS.iter().map(|p| p.name).collect();
        let unique_names: std::collections::HashSet<_> = names.iter().cloned().collect();

        assert_eq!(
            names.len(),
            unique_names.len(),
            "All prompt names should be unique"
        );
    }

    #[test]
    fn test_prompt_content_length() {
        for prompt in PROMPTS.iter() {
            assert!(
                prompt.content.len() > 10,
                "Prompt '{}' should have meaningful content",
                prompt.name
            );
        }
    }
}
