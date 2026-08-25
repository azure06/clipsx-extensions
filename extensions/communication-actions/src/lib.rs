mod bindings {
    use super::CommunicationActions;
    wit_bindgen::generate!({ path: "../../sdk/wit", world: "extension" });
    export!(CommunicationActions);
}

use bindings::clipsx::extension::types::{
    ActionResult, ActionState, CompactModel, Content, Facet, GuestError, GuestErrorCode,
    OutputRepresentation, RenderModel, Representation,
};

struct CommunicationActions;

impl bindings::Guest for CommunicationActions {
    fn detect(contribution_id: String, input: Representation) -> Result<Vec<Facet>, GuestError> {
        if contribution_id != "detect-contact" {
            return Ok(Vec::new());
        }
        let Content::Text(text) = input.content else {
            return Ok(Vec::new());
        };
        let candidate = text.trim();
        if let Some(address) = email(candidate) {
            return Ok(vec![Facet {
                id: "email".into(),
                payload_json: serde_json::json!({
                    "schemaVersion": 1,
                    "address": address,
                })
                .to_string(),
            }]);
        }
        if let Some(number) = phone(candidate) {
            return Ok(vec![Facet {
                id: "phone".into(),
                payload_json: serde_json::json!({
                    "schemaVersion": 1,
                    "number": number,
                })
                .to_string(),
            }]);
        }
        Ok(Vec::new())
    }

    fn render_detail(_: String, _: Representation, _: Option<Facet>) -> Result<RenderModel, GuestError> {
        Err(unsupported("Communication Actions has no custom renderer"))
    }
    fn render_compact(_: String, _: Representation, _: Option<Facet>) -> Result<CompactModel, GuestError> {
        Err(unsupported("Communication Actions uses the host compact summary"))
    }
    fn transform(_: String, _: Representation, _: String) -> Result<Vec<OutputRepresentation>, GuestError> {
        Err(unsupported("Communication Actions has no transformer"))
    }
    fn run_action(_: String, _: Representation, _: Option<Facet>, _: String) -> Result<ActionResult, GuestError> {
        Err(unsupported("Native actions are executed by the host"))
    }
    fn action_state(_: String, _: Representation, _: Option<Facet>, _: String) -> Result<ActionState, GuestError> {
        Ok(ActionState::Enabled)
    }
}

fn email(value: &str) -> Option<&str> {
    let (local, domain) = value.split_once('@')?;
    (!local.is_empty()
        && !domain.is_empty()
        && !domain.contains('@')
        && value.len() <= 320
        && !value.chars().any(char::is_whitespace))
    .then_some(value)
}

fn phone(value: &str) -> Option<&str> {
    let digits = value.chars().filter(char::is_ascii_digit).count();
    ((3..=15).contains(&digits)
        && value.len() <= 64
        && value.chars().all(|character| {
            character.is_ascii_digit() || matches!(character, '+' | '-' | '(' | ')' | ' ' | '.')
        }))
    .then_some(value)
}

fn unsupported(message: &str) -> GuestError {
    GuestError { code: GuestErrorCode::Unsupported, message: message.into() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_bounded_contact_values() {
        assert_eq!(email("hello@example.com"), Some("hello@example.com"));
        assert_eq!(phone("+81 90-1234-5678"), Some("+81 90-1234-5678"));
        assert_eq!(email("not an email"), None);
        assert_eq!(phone("123;shutdown"), None);
    }
}
