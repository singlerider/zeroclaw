use anyhow::Result;

pub fn normalize_expression(expression: &str) -> Result<String> {
    let expression = expression.trim();
    let field_count = expression.split_whitespace().count();
    match field_count {
        5 => {
            let fields: Vec<&str> = expression.split_whitespace().collect();
            Ok(format!("0 {} {} {} {} {}", fields[0], fields[1], fields[2], fields[3], fields[4]))
        }
        6 | 7 => Ok(expression.to_string()),
        _ => anyhow::bail!("Invalid cron expression: expected 5, 6, or 7 fields, got {field_count}"),
    }
}
