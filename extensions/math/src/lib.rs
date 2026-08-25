mod bindings {
    use super::Math;
    wit_bindgen::generate!({ path: "../../sdk/wit", world: "extension" });
    export!(Math);
}

use bindings::clipsx::extension::types::{
    ActionResult, ActionState, CompactModel, Content, Facet, GuestError, GuestErrorCode,
    OutputRepresentation, RenderModel, Representation,
};

struct Math;

impl bindings::Guest for Math {
    fn detect(id: String, input: Representation) -> Result<Vec<Facet>, GuestError> {
        if id != "detect-math" {
            return Ok(Vec::new());
        }
        let Content::Text(text) = input.content else {
            return Ok(Vec::new());
        };
        let Some((formula, display)) = formula(&text) else {
            return Ok(Vec::new());
        };
        Ok(vec![Facet {
            id: "math".into(),
            payload_json:
                serde_json::json!({ "schemaVersion": 1, "formula": formula, "display": display })
                    .to_string(),
        }])
    }
    fn render_detail(
        _: String,
        _: Representation,
        _: Option<Facet>,
    ) -> Result<RenderModel, GuestError> {
        Err(unsupported(
            "Math detail is provided by isolated package UI",
        ))
    }
    fn render_compact(
        _: String,
        _: Representation,
        _: Option<Facet>,
    ) -> Result<CompactModel, GuestError> {
        Err(unsupported("Math uses the host compact summary"))
    }
    fn transform(
        _: String,
        _: Representation,
        _: String,
    ) -> Result<Vec<OutputRepresentation>, GuestError> {
        Err(unsupported("Math has no transformer"))
    }
    fn run_action(
        _: String,
        _: Representation,
        _: Option<Facet>,
        _: String,
    ) -> Result<ActionResult, GuestError> {
        Err(unsupported("Math has no guest action"))
    }
    fn action_state(
        _: String,
        _: Representation,
        _: Option<Facet>,
        _: String,
    ) -> Result<ActionState, GuestError> {
        Ok(ActionState::Hidden)
    }
}

fn formula(source: &str) -> Option<(&str, bool)> {
    let value = source.trim();
    if value.len() > 32_768 {
        return None;
    }
    for (open, close, display) in [
        ("$$", "$$", true),
        ("\\[", "\\]", true),
        ("\\(", "\\)", false),
    ] {
        if let Some(body) = value
            .strip_prefix(open)
            .and_then(|value| value.strip_suffix(close))
        {
            let body = body.trim();
            return (!body.is_empty()).then_some((body, display));
        }
    }
    let structural = [
        "\\frac",
        "\\sqrt",
        "\\sum",
        "\\prod",
        "\\int",
        "\\lim",
        "\\begin{",
        "\\left",
        "\\mathbf",
        "\\mathbb",
        "\\overline",
    ];
    (value.contains('=') || structural.iter().any(|token| value.contains(token)))
        .then_some((value, true))
}

fn unsupported(message: &str) -> GuestError {
    GuestError {
        code: GuestErrorCode::Unsupported,
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn recognizes_delimited_and_structural_math() {
        assert_eq!(formula("$$ E = mc^2 $$"), Some(("E = mc^2", true)));
        assert_eq!(formula(r"\(x^2 + y^2\)"), Some(("x^2 + y^2", false)));
        assert!(formula(r"\frac{-b \pm \sqrt{b^2-4ac}}{2a}").is_some());
    }
    #[test]
    fn rejects_ordinary_prose_and_empty_delimiters() {
        assert!(formula("The total is $25 today.").is_none());
        assert!(formula("$$   $$").is_none());
        assert!(formula("a short note").is_none());
    }
}
