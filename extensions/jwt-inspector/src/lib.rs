mod bindings {
    use super::JwtInspector;
    wit_bindgen::generate!({ path: "../../sdk/wit", world: "extension" });
    export!(JwtInspector);
}

use bindings::clipsx::extension::types::*;

struct JwtInspector;

impl bindings::Guest for JwtInspector {
    fn detect(id: String, input: Representation) -> Result<Vec<Facet>, GuestError> {
        if id != "detect-jwt" {
            return Ok(vec![]);
        }
        let Some(token) = text(&input).map(str::trim) else {
            return Ok(vec![]);
        };
        let Some((header, payload)) = decode_token(token) else {
            return Ok(vec![]);
        };
        Ok(vec![Facet {
            id: "jwt".into(),
            payload_json: serde_json::json!({
                "schemaVersion": 1,
                "algorithm": header.get("alg").and_then(|v| v.as_str()),
                "tokenType": header.get("typ").and_then(|v| v.as_str()),
                "issuer": payload.get("iss").and_then(|v| v.as_str()),
                "subject": payload.get("sub").and_then(|v| v.as_str()),
                "expiresAt": payload.get("exp").and_then(|v| v.as_i64()),
                "verified": false
            })
            .to_string(),
        }])
    }

    fn render_detail(
        _: String,
        _: Representation,
        _: Option<Facet>,
    ) -> Result<RenderModel, GuestError> {
        Err(unsupported("JWT uses its custom detail view"))
    }
    fn render_compact(
        _: String,
        _: Representation,
        _: Option<Facet>,
    ) -> Result<CompactModel, GuestError> {
        Err(unsupported(
            "JWT Inspector does not replace history previews",
        ))
    }
    fn transform(
        id: String,
        input: Representation,
        parameters: String,
    ) -> Result<Vec<OutputRepresentation>, GuestError> {
        if id != "extract-jwt" {
            return Err(unsupported("unknown transformer"));
        }
        let (header, payload) = decode_token(text(&input).unwrap_or_default().trim())
            .ok_or_else(|| invalid("invalid JWT"))?;
        let params: serde_json::Value =
            serde_json::from_str(&parameters).map_err(|_| invalid("invalid parameters"))?;
        let value = match params.get("part").and_then(|v| v.as_str()) {
            Some("header") => header,
            Some("payload") => payload,
            _ => return Err(invalid("part must be header or payload")),
        };
        output(serde_json::to_string_pretty(&value).map_err(|_| failed("could not serialize JWT"))?)
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

fn text(input: &Representation) -> Option<&str> {
    if let Content::Text(v) = &input.content {
        Some(v)
    } else {
        None
    }
}
fn decode_token(token: &str) -> Option<(serde_json::Value, serde_json::Value)> {
    if token.len() > 1024 * 1024 {
        return None;
    }
    let mut parts = token.split('.');
    let h = parts.next()?;
    let p = parts.next()?;
    let s = parts.next()?;
    if parts.next().is_some() || h.is_empty() || p.is_empty() || s.is_empty() {
        return None;
    }
    let header = serde_json::from_slice::<serde_json::Value>(&decode_url(h)?).ok()?;
    let payload = serde_json::from_slice::<serde_json::Value>(&decode_url(p)?).ok()?;
    (header.is_object() && payload.is_object()).then_some((header, payload))
}
fn decode_url(value: &str) -> Option<Vec<u8>> {
    decode64(value.as_bytes(), true)
}
fn decode64(input: &[u8], url: bool) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    let mut acc = 0u32;
    let mut bits = 0u8;
    for &b in input {
        if b == b'=' {
            break;
        }
        let v = match b {
            b'A'..=b'Z' => b - b'A',
            b'a'..=b'z' => b - b'a' + 26,
            b'0'..=b'9' => b - b'0' + 52,
            b'-' if url => 62,
            b'_' if url => 63,
            b'+' => 62,
            b'/' => 63,
            _ => return None,
        };
        acc = (acc << 6) | v as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
            acc &= (1 << bits) - 1;
        }
    }
    Some(out)
}
fn output(value: String) -> Result<Vec<OutputRepresentation>, GuestError> {
    Ok(vec![OutputRepresentation {
        format_key: "mime:application/json".into(),
        mime_type: "application/json".into(),
        content: OutputContent::Text(value),
    }])
}
fn error(code: GuestErrorCode, message: &str) -> GuestError {
    GuestError {
        code,
        message: message.into(),
    }
}
fn invalid(m: &str) -> GuestError {
    error(GuestErrorCode::InvalidInput, m)
}
fn unsupported(m: &str) -> GuestError {
    error(GuestErrorCode::Unsupported, m)
}
fn failed(m: &str) -> GuestError {
    error(GuestErrorCode::Failed, m)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bindings::Guest as _;
    #[test]
    fn accepts_structural_jwt_without_claiming_verification() {
        let t = "eyJhbGciOiJub25lIiwidHlwIjoiSldUIn0.eyJzdWIiOiIxMjMifQ.signature";
        let (h, p) = decode_token(t).unwrap();
        assert_eq!(h["alg"], "none");
        assert_eq!(p["sub"], "123");
    }
    #[test]
    fn rejects_wrong_segments_and_json() {
        assert!(decode_token("a.b").is_none());
        assert!(decode_token("abc.def.ghi").is_none());
    }

    #[test]
    fn extracts_payload_as_pretty_json_for_copy() {
        let input = Representation {
            format_key: "mime:text/plain".into(),
            mime_type: Some("text/plain".into()),
            storage_kind: "text".into(),
            content: Content::Text(
                "eyJhbGciOiJub25lIiwidHlwIjoiSldUIn0.eyJzdWIiOiIxMjMiLCJhZG1pbiI6dHJ1ZX0.signature"
                    .into(),
            ),
        };
        let outputs = JwtInspector::transform(
            "extract-jwt".into(),
            input,
            serde_json::json!({"part":"payload"}).to_string(),
        )
        .unwrap();

        assert_eq!(outputs.len(), 1);
        assert_eq!(outputs[0].format_key, "mime:application/json");
        assert_eq!(outputs[0].mime_type, "application/json");
        assert!(matches!(
            &outputs[0].content,
            OutputContent::Text(value) if value == "{\n  \"admin\": true,\n  \"sub\": \"123\"\n}"
        ));
    }
}
