use serde_json::Value;

pub fn to_response_format(schema: &Value) -> Value {
    if let Some(obj) = schema.as_object() {
        if obj.contains_key("type") {
            return schema.clone();
        }
        return serde_json::json!({
            "type": "json_schema",
            "json_schema": {
                "name": "response",
                "strict": true,
                "schema": schema
            }
        });
    }
    serde_json::json!({"type": "json_object"})
}

pub fn parse_json_content(content: &str, _schema: &Value) -> Option<Value> {
    serde_json::from_str(content).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_response_format_with_type() {
        let schema = serde_json::json!({"type": "json_object"});
        let result = to_response_format(&schema);
        assert_eq!(result["type"], "json_object");
    }

    #[test]
    fn test_to_response_format_with_schema() {
        let schema = serde_json::json!({
            "properties": {"name": {"type": "string"}},
            "required": ["name"]
        });
        let result = to_response_format(&schema);
        assert_eq!(result["type"], "json_schema");
    }

    #[test]
    fn test_parse_json_content_valid() {
        let content = r#"{"name": "test"}"#;
        let schema = serde_json::json!({});
        let result = parse_json_content(content, &schema);
        assert!(result.is_some());
        assert_eq!(result.unwrap()["name"], "test");
    }

    #[test]
    fn test_parse_json_content_invalid() {
        let result = parse_json_content("not json", &serde_json::json!({}));
        assert!(result.is_none());
    }
}
