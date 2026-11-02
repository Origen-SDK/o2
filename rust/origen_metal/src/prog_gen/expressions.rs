//! Helpers for the raw tester expressions used by IfExpr and UnlessExpr flow conditions

/// Returns the flow variables read by the given tester expression.
///
/// Literals (numbers, strings, true/false), function names and the parts of dotted
/// references (e.g. a sub-flow's variable) are not flow variables of the current flow.
pub fn expression_variables(expression: &str) -> Vec<String> {
    let chars: Vec<char> = expression.chars().collect();
    let mut vars: Vec<String> = vec![];
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '"' || c == '\'' {
            i += 1;
            while i < chars.len() && chars[i] != c {
                if chars[i] == '\\' {
                    i += 1;
                }
                i += 1;
            }
            i += 1;
        } else if c.is_ascii_digit() {
            while i < chars.len()
                && (chars[i].is_ascii_alphanumeric() || chars[i] == '_' || chars[i] == '.')
            {
                i += 1;
            }
        } else if c.is_ascii_alphabetic() || c == '_' {
            let start = i;
            while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let name: String = chars[start..i].iter().collect();
            let prev = chars[..start].iter().rev().find(|c| !c.is_whitespace());
            let next = chars[i..].iter().find(|c| !c.is_whitespace());
            let is_literal = name == "true" || name == "false";
            let is_dotted = prev == Some(&'.') || next == Some(&'.');
            let is_function = next == Some(&'(');
            if !is_literal && !is_dotted && !is_function && !vars.contains(&name) {
                vars.push(name);
            }
        } else {
            i += 1;
        }
    }
    vars
}

#[cfg(test)]
mod tests {
    use super::expression_variables;

    #[test]
    fn expression_variables_excludes_literals_functions_and_references() {
        assert_eq!(expression_variables("mode == 6"), vec!["mode"]);
        assert_eq!(
            expression_variables("captureMode == 6 && (Other_var != 1.5e3)"),
            vec!["captureMode", "Other_var"]
        );
        assert_eq!(
            expression_variables("abs(delta) > 2 && sub.result == true && name == \"a b\""),
            vec!["delta", "name"]
        );
        assert_eq!(expression_variables("count + count"), vec!["count"]);
    }
}
