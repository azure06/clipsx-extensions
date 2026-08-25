mod bindings {
    use super::CurlToFetch;
    wit_bindgen::generate!({ path: "../../sdk/wit", world: "extension" });
    export!(CurlToFetch);
}
use bindings::clipsx::extension::types::*;
struct CurlToFetch;
impl bindings::Guest for CurlToFetch {
    fn detect(id: String, input: Representation) -> Result<Vec<Facet>, GuestError> {
        if id != "detect-curl" {
            return Ok(vec![]);
        }
        let Some(raw) = text(&input).map(str::trim) else {
            return Ok(vec![]);
        };
        let Ok(parsed) = parse(raw) else {
            return Ok(vec![]);
        };
        Ok(vec![Facet{id:"curl".into(),payload_json:serde_json::json!({"schemaVersion":1,"method":parsed.method,"url":parsed.url,"headerCount":parsed.headers.len(),"hasBody":parsed.body.is_some(),"warnings":parsed.warnings}).to_string()}])
    }
    fn render_detail(
        _: String,
        _: Representation,
        _: Option<Facet>,
    ) -> Result<RenderModel, GuestError> {
        Err(unsupported("curl uses its custom view"))
    }
    fn render_compact(
        id: String,
        input: Representation,
        _: Option<Facet>,
    ) -> Result<CompactModel, GuestError> {
        if id != "fetch-detail" {
            return Err(unsupported("unknown renderer"));
        }
        let p = parse(text(&input).unwrap_or_default()).map_err(|m| invalid(&m))?;
        Ok(CompactModel {
            leading: LeadingVisual::Monogram("ƒ".into()),
            title: Some(format!("{} {}", p.method, p.url)),
            subtitle: Some(format!(
                "{} headers{}",
                p.headers.len(),
                if p.body.is_some() { " · body" } else { "" }
            )),
            badge: Some("fetch".into()),
            accessibility_label: format!("curl converted to fetch for {}", p.url),
        })
    }
    fn transform(
        id: String,
        input: Representation,
        _: String,
    ) -> Result<Vec<OutputRepresentation>, GuestError> {
        if id != "convert-curl" {
            return Err(unsupported("unknown transformer"));
        }
        let p = parse(text(&input).unwrap_or_default()).map_err(|m| invalid(&m))?;
        Ok(vec![OutputRepresentation {
            format_key: "mime:text/javascript".into(),
            mime_type: "text/javascript".into(),
            content: OutputContent::Text(to_fetch(&p)),
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
#[derive(Debug)]
struct Parsed {
    url: String,
    method: String,
    headers: Vec<(String, String)>,
    body: Option<String>,
    warnings: Vec<String>,
}
fn parse(raw: &str) -> Result<Parsed, String> {
    if raw.len() > 1024 * 1024 {
        return Err("command is too large".into());
    }
    let words = words(raw)?;
    if words.first().map(|v| v.to_ascii_lowercase()) != Some("curl".into()) {
        return Err("command must start with curl".into());
    }
    let (mut url, mut method, mut headers, mut body, mut warnings): (
        Option<String>,
        String,
        Vec<(String, String)>,
        Option<String>,
        Vec<String>,
    ) = (None, "GET".to_string(), vec![], None, vec![]);
    let mut i = 1;
    while i < words.len() {
        match words[i].as_str() {
            "-X" | "--request" => {
                i += 1;
                method = words
                    .get(i)
                    .ok_or("request method is missing")?
                    .to_uppercase()
            }
            "-H" | "--header" => {
                i += 1;
                let h = words.get(i).ok_or("header is missing")?;
                let (k, v) = h.split_once(':').ok_or("header must contain a colon")?;
                headers.push((k.trim().into(), v.trim().into()))
            }
            "-d" | "--data" | "--data-raw" | "--data-binary" => {
                i += 1;
                body = Some(words.get(i).ok_or("request body is missing")?.clone());
                if method == "GET" {
                    method = "POST".into()
                }
            }
            "-u" | "--user" => {
                i += 1;
                let _ = words.get(i).ok_or("credentials are missing")?;
                warnings.push("Authorization credentials were omitted".into())
            }
            v if v.starts_with('-') => warnings.push(format!("Unsupported flag: {v}")),
            v => {
                if url.is_none() {
                    url = Some(v.into())
                } else {
                    warnings.push(format!("Ignored argument: {v}"))
                }
            }
        }
        i += 1
    }
    let url = url.ok_or("curl URL is missing")?;
    if !(url.starts_with("https://") || url.starts_with("http://")) {
        return Err("only HTTP(S) curl URLs are supported".into());
    }
    Ok(Parsed {
        url,
        method,
        headers,
        body,
        warnings,
    })
}
fn words(raw: &str) -> Result<Vec<String>, String> {
    let mut out = vec![];
    let (mut current, mut quote, mut escape) = (String::new(), None, false);
    for ch in raw.replace("\\\r\n", " ").replace("\\\n", " ").chars() {
        if escape {
            current.push(ch);
            escape = false;
            continue;
        }
        if ch == '\\' && quote != Some('\'') {
            escape = true;
            continue;
        }
        if matches!(ch, '\'' | '"') {
            if quote == Some(ch) {
                quote = None
            } else if quote.is_none() {
                quote = Some(ch)
            } else {
                current.push(ch)
            }
            continue;
        }
        if ch.is_whitespace() && quote.is_none() {
            if !current.is_empty() {
                out.push(std::mem::take(&mut current))
            }
        } else {
            current.push(ch)
        }
    }
    if quote.is_some() {
        return Err("curl command has an unclosed quote".into());
    }
    if !current.is_empty() {
        out.push(current)
    }
    Ok(out)
}
fn js(v: &str) -> String {
    serde_json::to_string(v).unwrap_or_else(|_| "\"\"".into())
}
fn to_fetch(p: &Parsed) -> String {
    let mut options = vec![];
    if p.method != "GET" {
        options.push(format!("  method: {}", js(&p.method)))
    }
    if !p.headers.is_empty() {
        let values = p
            .headers
            .iter()
            .map(|(k, v)| format!("    {}: {}", js(k), js(v)))
            .collect::<Vec<_>>()
            .join(",\n");
        options.push(format!("  headers: {{\n{values}\n  }}"))
    }
    if let Some(body) = &p.body {
        options.push(format!("  body: {}", js(body)))
    }
    if options.is_empty() {
        format!(
            "const response = await fetch({})\nconst data = await response.json()",
            js(&p.url)
        )
    } else {
        format!(
            "const response = await fetch({}, {{\n{}\n}})\nconst data = await response.json()",
            js(&p.url),
            options.join(",\n")
        )
    }
}
fn text(i: &Representation) -> Option<&str> {
    if let Content::Text(v) = &i.content {
        Some(v)
    } else {
        None
    }
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
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn converts_common_curl() {
        let p=parse("curl -X POST -H 'Content-Type: application/json' -d '{\"ok\":true}' https://example.com/api").unwrap();
        let out = to_fetch(&p);
        assert!(out.contains("POST"));
        assert!(out.contains("Content-Type"));
    }
    #[test]
    fn warns_without_leaking_credentials() {
        let p = parse("curl -u user:secret https://example.com").unwrap();
        assert!(!to_fetch(&p).contains("secret"));
        assert_eq!(p.warnings.len(), 1)
    }
}
