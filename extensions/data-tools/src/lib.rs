mod bindings {
    use super::DataTools;
    wit_bindgen::generate!({ path: "../../sdk/wit", world: "extension" });
    export!(DataTools);
}
use bindings::clipsx::extension::types::*;
struct DataTools;
impl bindings::Guest for DataTools {
    fn detect(_: String, _: Representation) -> Result<Vec<Facet>, GuestError> {
        Ok(vec![])
    }
    fn render_detail(
        _: String,
        _: Representation,
        _: Option<Facet>,
    ) -> Result<RenderModel, GuestError> {
        Err(unsupported("Data Tools is action-only"))
    }
    fn render_compact(
        _: String,
        _: Representation,
        _: Option<Facet>,
    ) -> Result<CompactModel, GuestError> {
        Err(unsupported("Data Tools has no compact renderer"))
    }
    fn transform(
        id: String,
        input: Representation,
        params: String,
    ) -> Result<Vec<OutputRepresentation>, GuestError> {
        if id != "data-transform" {
            return Err(unsupported("unknown transformer"));
        }
        let raw = text(&input).ok_or_else(|| invalid("text input required"))?;
        if raw.len() > 10 * 1024 * 1024 {
            return Err(invalid("input exceeds 10 MiB"));
        }
        let p: serde_json::Value =
            serde_json::from_str(&params).map_err(|_| invalid("invalid parameters"))?;
        let op = p
            .get("operation")
            .and_then(|v| v.as_str())
            .ok_or_else(|| invalid("operation is required"))?;
        let (value, mime) = match op {
            "format-json" => (
                serde_json::to_string_pretty(&json(raw)?)
                    .map_err(|_| failed("could not format JSON"))?,
                "application/json",
            ),
            "minify-json" => (
                serde_json::to_string(&json(raw)?).map_err(|_| failed("could not minify JSON"))?,
                "application/json",
            ),
            "json-to-csv" => (json_to_csv(&json(raw)?)?, "text/csv"),
            "json-to-markdown" => (rows_to_markdown(json_rows(&json(raw)?)?)?, "text/markdown"),
            "csv-to-json" => {
                let rows = parse_csv(raw)?;
                (
                    serde_json::to_string_pretty(&csv_objects(&rows)?)
                        .map_err(|_| failed("could not serialize JSON"))?,
                    "application/json",
                )
            }
            "csv-to-markdown" => (rows_to_markdown(parse_csv(raw)?)?, "text/markdown"),
            _ => return Err(invalid("unsupported data operation")),
        };
        Ok(vec![OutputRepresentation {
            format_key: format!("mime:{mime}"),
            mime_type: mime.into(),
            content: OutputContent::Text(value),
        }])
    }
    fn run_action(
        _: String,
        _: Representation,
        _: Option<Facet>,
        _: String,
    ) -> Result<ActionResult, GuestError> {
        Err(unsupported("actions use transformer presets"))
    }
    fn action_state(
        _: String,
        _: Representation,
        _: Option<Facet>,
        _: String,
    ) -> Result<ActionState, GuestError> {
        Ok(ActionState::Enabled)
    }
}
fn text(i: &Representation) -> Option<&str> {
    if let Content::Text(v) = &i.content {
        Some(v)
    } else {
        None
    }
}
fn json(raw: &str) -> Result<serde_json::Value, GuestError> {
    serde_json::from_str(raw).map_err(|_| invalid("input is not valid JSON"))
}
fn scalar(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::Null => String::new(),
        serde_json::Value::String(s) => s.clone(),
        _ => v.to_string(),
    }
}
fn json_rows(value: &serde_json::Value) -> Result<Vec<Vec<String>>, GuestError> {
    let rows = value
        .as_array()
        .ok_or_else(|| invalid("JSON conversion requires an array"))?;
    if rows.is_empty() {
        return Err(invalid("JSON array is empty"));
    }
    if rows.iter().all(|v| v.is_object()) {
        let mut keys = vec![];
        for row in rows {
            for key in row.as_object().unwrap().keys() {
                if !keys.contains(key) {
                    keys.push(key.clone())
                }
            }
        }
        let mut out = vec![keys.clone()];
        for row in rows {
            let obj = row.as_object().unwrap();
            out.push(
                keys.iter()
                    .map(|k| obj.get(k).map(scalar).unwrap_or_default())
                    .collect(),
            )
        }
        Ok(out)
    } else if rows.iter().all(|v| v.is_array()) {
        Ok(rows
            .iter()
            .map(|r| r.as_array().unwrap().iter().map(scalar).collect())
            .collect())
    } else {
        Err(invalid("JSON array must contain objects or arrays"))
    }
}
fn csv_cell(v: &str) -> String {
    if v.chars().any(|ch| matches!(ch, ',' | '"' | '\n' | '\r')) {
        format!("\"{}\"", v.replace('"', "\"\""))
    } else {
        v.into()
    }
}
fn json_to_csv(v: &serde_json::Value) -> Result<String, GuestError> {
    Ok(json_rows(v)?
        .into_iter()
        .map(|r| r.iter().map(|v| csv_cell(v)).collect::<Vec<_>>().join(","))
        .collect::<Vec<_>>()
        .join("\r\n"))
}
fn parse_csv(raw: &str) -> Result<Vec<Vec<String>>, GuestError> {
    let (mut rows, mut row, mut cell, mut quoted) = (vec![], vec![], String::new(), false);
    let mut chars = raw.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '"' if quoted && chars.peek() == Some(&'"') => {
                cell.push('"');
                chars.next();
            }
            '"' => quoted = !quoted,
            ',' if !quoted => row.push(std::mem::take(&mut cell)),
            '\n' if !quoted => {
                row.push(std::mem::take(&mut cell));
                if row.iter().any(|v| !v.is_empty()) {
                    rows.push(std::mem::take(&mut row))
                }
            }
            '\r' if !quoted => {}
            _ => cell.push(ch),
        }
    }
    if quoted {
        return Err(invalid("CSV has an unclosed quote"));
    }
    row.push(cell);
    if row.iter().any(|v| !v.is_empty()) {
        rows.push(row)
    }
    if rows.is_empty() {
        return Err(invalid("CSV is empty"));
    }
    let width = rows[0].len();
    if width == 0 || rows.iter().any(|r| r.len() != width) {
        return Err(invalid("CSV rows have inconsistent columns"));
    }
    Ok(rows)
}
fn csv_objects(rows: &[Vec<String>]) -> Result<serde_json::Value, GuestError> {
    if rows.len() < 2 {
        return Err(invalid("CSV requires a header and at least one row"));
    }
    let h = &rows[0];
    Ok(serde_json::Value::Array(
        rows[1..]
            .iter()
            .map(|r| {
                serde_json::Value::Object(
                    h.iter()
                        .cloned()
                        .zip(r.iter().cloned().map(serde_json::Value::String))
                        .collect(),
                )
            })
            .collect(),
    ))
}
fn md(v: &str) -> String {
    v.replace('|', "\\|").replace('\n', "<br>")
}
fn rows_to_markdown(rows: Vec<Vec<String>>) -> Result<String, GuestError> {
    if rows.is_empty() {
        return Err(invalid("table is empty"));
    }
    let width = rows[0].len();
    let mut out = String::new();
    out.push_str(&format!(
        "| {} |\n",
        rows[0]
            .iter()
            .map(|v| md(v))
            .collect::<Vec<_>>()
            .join(" | ")
    ));
    out.push_str(&format!("| {} |\n", vec!["---"; width].join(" | ")));
    for row in &rows[1..] {
        out.push_str(&format!(
            "| {} |\n",
            row.iter().map(|v| md(v)).collect::<Vec<_>>().join(" | ")
        ))
    }
    Ok(out)
}
fn err(c: GuestErrorCode, m: &str) -> GuestError {
    GuestError {
        code: c,
        message: m.into(),
    }
}
fn invalid(m: &str) -> GuestError {
    err(GuestErrorCode::InvalidInput, m)
}
fn unsupported(m: &str) -> GuestError {
    err(GuestErrorCode::Unsupported, m)
}
fn failed(m: &str) -> GuestError {
    err(GuestErrorCode::Failed, m)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn converts_object_array() {
        let v = json("[{\"a\":1,\"b\":\"x\"},{\"a\":2,\"b\":\"y\"}]").unwrap();
        assert_eq!(json_to_csv(&v).unwrap(), "a,b\r\n1,x\r\n2,y");
    }
    #[test]
    fn parses_quoted_csv() {
        let rows = parse_csv("name,note\nA,\"x,y\"").unwrap();
        assert_eq!(rows[1][1], "x,y");
        assert!(rows_to_markdown(rows).unwrap().contains("| A | x,y |"));
    }
}
