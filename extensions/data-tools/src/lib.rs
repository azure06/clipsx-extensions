mod bindings {
    use super::DataTools;
    wit_bindgen::generate!({ path: "../../sdk/wit", world: "extension" });
    export!(DataTools);
}

use bindings::clipsx::extension::types::*;
use serde_json::{Map, Value};
use std::collections::BTreeSet;

const MAX_INPUT_BYTES: usize = 10 * 1024 * 1024;
struct DataTools;

impl bindings::Guest for DataTools {
    fn detect(id: String, input: Representation) -> Result<Vec<Facet>, GuestError> {
        if id != "markdown-table-detector" {
            return Err(unsupported("unknown detector"));
        }
        let Some(raw) = text(&input) else {
            return Ok(vec![]);
        };
        Ok(parse_markdown_table(raw)
            .ok()
            .map(|_| {
                vec![Facet {
                    id: "markdown-table".into(),
                    payload_json: serde_json::json!({"schemaVersion": 1, "kind": "markdown-table"})
                        .to_string(),
                }]
            })
            .unwrap_or_default())
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
        let raw = text(&input).ok_or_else(|| invalid("Data Tools requires text input"))?;
        if raw.len() > MAX_INPUT_BYTES {
            return Err(invalid("input exceeds the 10 MiB Data Tools limit"));
        }
        let params: Value =
            serde_json::from_str(&params).map_err(|_| invalid("parameters must be valid JSON"))?;
        let operation = params
            .get("operation")
            .and_then(Value::as_str)
            .ok_or_else(|| invalid("operation is required"))?;
        let root_name = params
            .get("root_name")
            .and_then(Value::as_str)
            .unwrap_or("Root");
        let (value, mime) = convert(operation, raw, root_name)?;
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
        Err(unsupported("Data Tools actions use transformer presets"))
    }
    fn action_state(
        id: String,
        input: Representation,
        _: Option<Facet>,
        _: String,
    ) -> Result<ActionState, GuestError> {
        if let Some(raw) = text(&input) {
            let first = raw
                .lines()
                .find(|line| !line.trim().is_empty())
                .unwrap_or("");
            let is_tsv = first.matches('\t').count() > first.matches(',').count();
            if (id == "table-to-csv" && !is_tsv) || (id == "table-to-tsv" && is_tsv) {
                return Ok(ActionState::Hidden);
            }
        }
        Ok(ActionState::Enabled)
    }
}

fn convert(
    operation: &str,
    raw: &str,
    root_name: &str,
) -> Result<(String, &'static str), GuestError> {
    match operation {
        "json-to-csv" => Ok((write_delimited(&json_rows(&json(raw)?)?, b',')?, "text/csv")),
        "json-to-tsv" => Ok((
            write_delimited(&json_rows(&json(raw)?)?, b'\t')?,
            "text/tab-separated-values",
        )),
        "json-to-markdown" => Ok((write_markdown(&json_rows(&json(raw)?)?), "text/markdown")),
        "json-to-yaml" => Ok((
            serde_yaml::to_string(&json(raw)?)
                .map_err(|e| failed(&format!("could not write YAML: {e}")))?,
            "application/yaml",
        )),
        "json-to-toml" => Ok((json_to_toml(&json(raw)?)?, "application/toml")),
        "json-to-typescript" => Ok((
            json_to_typescript(&json(raw)?, root_name)?,
            "text/typescript",
        )),
        "table-to-json" => Ok((
            pretty_json(&rows_to_objects(&parse_delimited_auto(raw)?)?)?,
            "application/json",
        )),
        "table-to-csv" => Ok((
            write_delimited(&parse_delimited_auto(raw)?, b',')?,
            "text/csv",
        )),
        "table-to-tsv" => Ok((
            write_delimited(&parse_delimited_auto(raw)?, b'\t')?,
            "text/tab-separated-values",
        )),
        "table-to-markdown" => Ok((write_markdown(&parse_delimited_auto(raw)?), "text/markdown")),
        "markdown-to-json" => Ok((
            pretty_json(&rows_to_objects(&parse_markdown_table(raw)?)?)?,
            "application/json",
        )),
        "markdown-to-csv" => Ok((
            write_delimited(&parse_markdown_table(raw)?, b',')?,
            "text/csv",
        )),
        "markdown-to-tsv" => Ok((
            write_delimited(&parse_markdown_table(raw)?, b'\t')?,
            "text/tab-separated-values",
        )),
        "yaml-to-json" => {
            let value: Value = serde_yaml::from_str(raw)
                .map_err(|e| invalid(&format!("input is not valid YAML: {e}")))?;
            Ok((pretty_json(&value)?, "application/json"))
        }
        "toml-to-json" => {
            let value: toml::Value = toml::from_str(raw)
                .map_err(|e| invalid(&format!("input is not valid TOML: {e}")))?;
            Ok((pretty_json(&value)?, "application/json"))
        }
        "url-encode" => Ok((urlencoding::encode(raw).into_owned(), "text/plain")),
        "url-decode" => Ok((
            urlencoding::decode(raw)
                .map_err(|e| invalid(&format!("input is not valid percent-encoded text: {e}")))?
                .into_owned(),
            "text/plain",
        )),
        "url-normalize" => Ok((normalize_url(raw)?, "text/plain")),
        "url-query-to-json" => Ok((url_query_to_json(raw)?, "application/json")),
        _ => Err(invalid("unsupported Data Tools operation")),
    }
}

fn text(input: &Representation) -> Option<&str> {
    if let Content::Text(value) = &input.content {
        Some(value)
    } else {
        None
    }
}
fn json(raw: &str) -> Result<Value, GuestError> {
    serde_json::from_str(raw).map_err(|e| invalid(&format!("input is not valid JSON: {e}")))
}
fn pretty_json<T: serde::Serialize>(value: &T) -> Result<String, GuestError> {
    serde_json::to_string_pretty(value).map_err(|e| failed(&format!("could not write JSON: {e}")))
}
fn scalar(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::String(value) => value.clone(),
        _ => value.to_string(),
    }
}

fn json_rows(value: &Value) -> Result<Vec<Vec<String>>, GuestError> {
    let values = value
        .as_array()
        .ok_or_else(|| invalid("table conversion requires a top-level JSON array"))?;
    if values.is_empty() {
        return Err(invalid("JSON array is empty"));
    }
    if values.iter().all(Value::is_object) {
        let mut headers = Vec::new();
        for value in values {
            for key in value.as_object().expect("object checked").keys() {
                if !headers.contains(key) {
                    headers.push(key.clone());
                }
            }
        }
        if headers.is_empty() {
            return Err(invalid("JSON objects contain no columns"));
        }
        let mut rows = vec![headers.clone()];
        rows.extend(values.iter().map(|value| {
            let object = value.as_object().expect("object checked");
            headers
                .iter()
                .map(|key| object.get(key).map(scalar).unwrap_or_default())
                .collect()
        }));
        Ok(rows)
    } else if values.iter().all(Value::is_array) {
        validate_rows(
            values
                .iter()
                .map(|value| {
                    value
                        .as_array()
                        .expect("array checked")
                        .iter()
                        .map(scalar)
                        .collect()
                })
                .collect(),
        )
    } else {
        Err(invalid(
            "JSON array must contain only objects or only arrays",
        ))
    }
}

fn parse_delimited_auto(raw: &str) -> Result<Vec<Vec<String>>, GuestError> {
    let first = raw
        .lines()
        .find(|line| !line.trim().is_empty())
        .ok_or_else(|| invalid("table is empty"))?;
    parse_delimited(
        raw,
        if first.matches('\t').count() > first.matches(',').count() {
            b'\t'
        } else {
            b','
        },
    )
}
fn parse_delimited(raw: &str, delimiter: u8) -> Result<Vec<Vec<String>>, GuestError> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(delimiter)
        .flexible(false)
        .from_reader(raw.as_bytes());
    let mut rows = Vec::new();
    for record in reader.records() {
        rows.push(
            record
                .map_err(|e| invalid(&format!("invalid delimited table: {e}")))?
                .iter()
                .map(str::to_owned)
                .collect(),
        );
    }
    validate_rows(rows)
}
fn validate_rows(rows: Vec<Vec<String>>) -> Result<Vec<Vec<String>>, GuestError> {
    if rows.is_empty() || rows[0].is_empty() {
        return Err(invalid("table is empty"));
    }
    let width = rows[0].len();
    if rows.iter().any(|row| row.len() != width) {
        return Err(invalid("table rows have inconsistent columns"));
    }
    Ok(rows)
}
fn write_delimited(rows: &[Vec<String>], delimiter: u8) -> Result<String, GuestError> {
    let mut writer = csv::WriterBuilder::new()
        .delimiter(delimiter)
        .terminator(csv::Terminator::CRLF)
        .from_writer(Vec::new());
    for row in rows {
        writer
            .write_record(row)
            .map_err(|e| failed(&format!("could not write table: {e}")))?;
    }
    let mut output = String::from_utf8(
        writer
            .into_inner()
            .map_err(|e| failed(&format!("could not finish table: {e}")))?,
    )
    .map_err(|_| failed("table output was not UTF-8"))?;
    while output.ends_with("\r\n") {
        output.truncate(output.len() - 2);
    }
    Ok(output)
}
fn rows_to_objects(rows: &[Vec<String>]) -> Result<Value, GuestError> {
    if rows.len() < 2 {
        return Err(invalid("table requires a header and at least one data row"));
    }
    if rows[0].iter().any(|header| header.trim().is_empty()) {
        return Err(invalid("table headers cannot be empty"));
    }
    let unique: BTreeSet<&String> = rows[0].iter().collect();
    if unique.len() != rows[0].len() {
        return Err(invalid("table headers must be unique"));
    }
    Ok(Value::Array(
        rows[1..]
            .iter()
            .map(|row| {
                Value::Object(
                    rows[0]
                        .iter()
                        .cloned()
                        .zip(row.iter().cloned().map(Value::String))
                        .collect(),
                )
            })
            .collect(),
    ))
}

fn write_markdown(rows: &[Vec<String>]) -> String {
    fn cell(value: &str) -> String {
        value
            .replace('\\', "\\\\")
            .replace('|', "\\|")
            .replace(['\r', '\n'], "<br>")
    }
    let line = |row: &[String]| {
        format!(
            "| {} |",
            row.iter()
                .map(|value| cell(value))
                .collect::<Vec<_>>()
                .join(" | ")
        )
    };
    let mut output = vec![
        line(&rows[0]),
        format!("| {} |", vec!["---"; rows[0].len()].join(" | ")),
    ];
    output.extend(rows[1..].iter().map(|row| line(row)));
    output.join("\n")
}
fn split_markdown_row(line: &str) -> Vec<String> {
    let mut cells = vec![String::new()];
    let mut escaped = false;
    for ch in line
        .trim()
        .trim_start_matches('|')
        .trim_end_matches('|')
        .chars()
    {
        if escaped {
            cells.last_mut().expect("cell").push(ch);
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if ch == '|' {
            cells.push(String::new());
        } else {
            cells.last_mut().expect("cell").push(ch);
        }
    }
    if escaped {
        cells.last_mut().expect("cell").push('\\');
    }
    cells
        .into_iter()
        .map(|cell| cell.trim().replace("<br>", "\n"))
        .collect()
}
fn parse_markdown_table(raw: &str) -> Result<Vec<Vec<String>>, GuestError> {
    let lines: Vec<&str> = raw.lines().filter(|line| !line.trim().is_empty()).collect();
    if lines.len() < 3 || !lines[0].contains('|') {
        return Err(invalid(
            "Markdown table requires a header, separator, and data row",
        ));
    }
    let headers = split_markdown_row(lines[0]);
    let separators = split_markdown_row(lines[1]);
    if separators.len() != headers.len()
        || separators.iter().any(|cell| {
            let value = cell.trim().trim_matches(':');
            value.len() < 3 || !value.chars().all(|ch| ch == '-')
        })
    {
        return Err(invalid("Markdown table has an invalid header separator"));
    }
    let mut rows = vec![headers];
    rows.extend(lines[2..].iter().map(|line| split_markdown_row(line)));
    validate_rows(rows)
}

fn json_to_toml(value: &Value) -> Result<String, GuestError> {
    if !value.is_object() {
        return Err(invalid("TOML conversion requires a top-level JSON object"));
    }
    toml::to_string_pretty(value)
        .map_err(|e| invalid(&format!("JSON contains a value TOML cannot represent: {e}")))
}
fn type_name(value: &str) -> Result<&str, GuestError> {
    let value = value.trim();
    if value.is_empty()
        || value.len() > 80
        || !value
            .chars()
            .next()
            .is_some_and(|ch| ch.is_ascii_alphabetic() || ch == '_')
        || !value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
    {
        Err(invalid(
            "root_name must be a valid TypeScript identifier of at most 80 characters",
        ))
    } else {
        Ok(value)
    }
}
fn json_to_typescript(value: &Value, root_name: &str) -> Result<String, GuestError> {
    fn ty(value: &Value, indent: usize) -> String {
        match value {
            Value::Null => "null".into(),
            Value::Bool(_) => "boolean".into(),
            Value::Number(_) => "number".into(),
            Value::String(_) => "string".into(),
            Value::Array(values) => {
                let mut types: Vec<String> = values.iter().map(|value| ty(value, indent)).collect();
                types.sort();
                types.dedup();
                format!(
                    "Array<{}>",
                    if types.is_empty() {
                        "unknown".into()
                    } else {
                        types.join(" | ")
                    }
                )
            }
            Value::Object(values) => {
                let pad = " ".repeat(indent + 2);
                let close = " ".repeat(indent);
                let fields = values
                    .iter()
                    .map(|(key, value)| {
                        format!(
                            "{pad}{}: {};",
                            serde_json::to_string(key).unwrap_or_else(|_| format!("\"{key}\"")),
                            ty(value, indent + 2)
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                format!("{{\n{fields}\n{close}}}")
            }
        }
    }
    Ok(format!(
        "export type {} = {};",
        type_name(root_name)?,
        ty(value, 0)
    ))
}

fn normalize_url(raw: &str) -> Result<String, GuestError> {
    let mut url = url::Url::parse(raw.trim())
        .map_err(|e| invalid(&format!("input is not a valid absolute URL: {e}")))?;
    url.set_fragment(None);
    let mut pairs: Vec<(String, String)> = url
        .query_pairs()
        .map(|(key, value)| (key.into_owned(), value.into_owned()))
        .collect();
    pairs.sort();
    url.set_query(None);
    if !pairs.is_empty() {
        url.query_pairs_mut().extend_pairs(pairs);
    }
    Ok(url.to_string())
}
fn url_query_to_json(raw: &str) -> Result<String, GuestError> {
    let url = url::Url::parse(raw.trim())
        .map_err(|e| invalid(&format!("input is not a valid absolute URL: {e}")))?;
    let mut values = Map::new();
    for (key, value) in url.query_pairs() {
        match values.get_mut(key.as_ref()) {
            Some(Value::Array(items)) => items.push(Value::String(value.into_owned())),
            Some(existing) => {
                let first = existing.take();
                *existing = Value::Array(vec![first, Value::String(value.into_owned())]);
            }
            None => {
                values.insert(key.into_owned(), Value::String(value.into_owned()));
            }
        }
    }
    pretty_json(&Value::Object(values))
}

fn error(code: GuestErrorCode, message: &str) -> GuestError {
    GuestError {
        code,
        message: message.into(),
    }
}
fn invalid(message: &str) -> GuestError {
    error(GuestErrorCode::InvalidInput, message)
}
fn unsupported(message: &str) -> GuestError {
    error(GuestErrorCode::Unsupported, message)
}
fn failed(message: &str) -> GuestError {
    error(GuestErrorCode::Failed, message)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn csv_round_trips_quotes_newlines_and_tabs() {
        let rows = parse_delimited("name,note\r\nAda,\"x,y\"\r\nLin,\"two\nlines\"", b',').unwrap();
        assert_eq!(rows[2][1], "two\nlines");
        assert!(write_delimited(&rows, b'\t').unwrap().contains("Ada\tx,y"));
    }
    #[test]
    fn json_table_converts_to_native_preview_formats() {
        let input = r#"[{"name":"Ada","active":true},{"name":"Lin","active":false}]"#;
        assert!(convert("json-to-csv", input, "Root")
            .unwrap()
            .0
            .contains(','));
        assert!(convert("json-to-tsv", input, "Root")
            .unwrap()
            .0
            .contains('\t'));
        assert!(convert("json-to-markdown", input, "Root")
            .unwrap()
            .0
            .contains("| --- |"));
    }
    #[test]
    fn markdown_is_strict_and_unescapes_cells() {
        assert!(parse_markdown_table("# Title\nParagraph").is_err());
        let rows =
            parse_markdown_table("| name | note |\n| --- | --- |\n| Ada | one\\|two |").unwrap();
        assert_eq!(rows[1][1], "one|two");
    }
    #[test]
    fn yaml_and_toml_preserve_nested_values() {
        assert!(
            convert("yaml-to-json", "name: Ada\nmeta:\n  active: true\n", "Root")
                .unwrap()
                .0
                .contains("\"active\": true")
        );
        assert!(convert(
            "toml-to-json",
            "name = \"Ada\"\n[meta]\nactive = true\n",
            "Root"
        )
        .unwrap()
        .0
        .contains("\"active\": true"));
        assert!(convert("json-to-toml", "[1,2]", "Root").is_err());
    }
    #[test]
    fn typescript_validates_name_and_models_shape() {
        let output = convert("json-to-typescript", r#"{"name":"Ada","age":37}"#, "Person")
            .unwrap()
            .0;
        assert!(output.starts_with("export type Person"));
        assert!(output.contains("\"age\": number"));
        assert!(convert("json-to-typescript", "{}", "not valid").is_err());
    }
    #[test]
    fn url_tools_handle_repeated_values() {
        assert_eq!(
            convert("url-decode", "hello%20world", "Root").unwrap().0,
            "hello world"
        );
        assert!(convert(
            "url-query-to-json",
            "https://example.com/?tag=b&tag=a",
            "Root"
        )
        .unwrap()
        .0
        .contains('['));
        assert_eq!(
            convert("url-normalize", "https://example.com/?z=2&a=1#x", "Root")
                .unwrap()
                .0,
            "https://example.com/?a=1&z=2"
        );
    }
    #[test]
    fn rejects_bad_table_shapes() {
        assert!(parse_delimited("", b',').is_err());
        assert!(parse_delimited("a,b\n1", b',').is_err());
        assert!(
            rows_to_objects(&[vec!["a".into(), "a".into()], vec!["1".into(), "2".into()]]).is_err()
        );
    }
}
