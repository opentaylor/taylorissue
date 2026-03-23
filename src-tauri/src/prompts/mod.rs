pub fn render(template: &str, vars: &[(&str, &str)]) -> String {
    let mut result = template.to_string();
    for (key, value) in vars {
        result = result.replace(&format!("{{{}}}", key), value);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_basic() {
        let tpl = "Hello {name}, port is {port}.";
        let out = render(tpl, &[("name", "Alice"), ("port", "8080")]);
        assert_eq!(out, "Hello Alice, port is 8080.");
    }

    #[test]
    fn test_render_no_vars() {
        let tpl = "No placeholders here.";
        assert_eq!(render(tpl, &[]), tpl);
    }

    #[test]
    fn test_render_missing_key() {
        let tpl = "Hello {name}, {unknown} stays.";
        let out = render(tpl, &[("name", "Bob")]);
        assert_eq!(out, "Hello Bob, {unknown} stays.");
    }

    #[test]
    fn test_render_multiple_occurrences() {
        let tpl = "{x} and {x} again";
        let out = render(tpl, &[("x", "hi")]);
        assert_eq!(out, "hi and hi again");
    }
}
