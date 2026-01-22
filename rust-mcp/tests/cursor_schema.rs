#[test]
fn cursor_can_parse_tool_schemas() {
    // Cursor is picky about union schemas like anyOf / oneOf / type arrays / $ref / definitions.
    // With declarative config, validate the schemas in our default tools.json seed.
    let raw = include_str!("../config-defaults/tools.json");
    let v: serde_json::Value = serde_json::from_str(raw).expect("parse default tools.json");
    let tools = v
        .get("tools")
        .and_then(|x| x.as_array())
        .expect("tools array");

    fn walk(schema: &serde_json::Value) {
        match schema {
            serde_json::Value::Object(map) => {
                for (k, v) in map {
                    assert_ne!(k, "anyOf", "schema contains anyOf");
                    assert_ne!(k, "oneOf", "schema contains oneOf");
                    assert_ne!(k, "allOf", "schema contains allOf");
                    assert_ne!(k, "$ref", "schema contains $ref");
                    assert_ne!(k, "definitions", "schema contains definitions");
                    if k == "type" {
                        assert!(!v.is_array(), "schema contains type array");
                    }
                    walk(v);
                }
            }
            serde_json::Value::Array(arr) => {
                for v in arr {
                    walk(v);
                }
            }
            _ => {}
        }
    }

    for t in tools {
        let schema = t
            .get("inputSchema")
            .unwrap_or_else(|| panic!("tool missing inputSchema: {t}"));
        walk(schema);
    }
}
