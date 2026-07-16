use std::collections::HashMap;

use super::{DiagnosticScope, EvaluationError, RequestDiagnostic};

pub(super) type ParsedHeaderValues = HashMap<http::HeaderName, String>;

pub(super) struct ParsedHeaders {
    pub values: ParsedHeaderValues,
    pub diagnostics: Vec<RequestDiagnostic>,
}

pub(super) fn parse_request_method(value: &str) -> Result<http::Method, EvaluationError> {
    http::Method::from_bytes(value.as_bytes())
        .map_err(|_| EvaluationError::InvalidRequestMethod(value.to_owned()))
}

pub(super) fn parse_headers(text: &str) -> ParsedHeaders {
    let mut values = HashMap::new();
    let mut first_lines = HashMap::<http::HeaderName, usize>::new();
    let mut diagnostics = Vec::new();

    for (offset, original) in text.lines().enumerate() {
        let line_number = offset + 1;
        if original.trim_matches([' ', '\t']).is_empty() {
            continue;
        }
        let Some((raw_name, raw_value)) = original.split_once(':') else {
            diagnostics.push(diagnostic(line_number, EvaluationError::MissingHeaderColon));
            continue;
        };
        let name_text = raw_name.trim_matches([' ', '\t']);
        let value_text = raw_value.trim_matches([' ', '\t']);
        let name = match http::HeaderName::from_bytes(name_text.as_bytes()) {
            Ok(name) => name,
            Err(_) => {
                diagnostics.push(diagnostic(
                    line_number,
                    EvaluationError::InvalidHeaderName(name_text.to_owned()),
                ));
                continue;
            }
        };
        let value = match http::HeaderValue::from_bytes(value_text.as_bytes()) {
            Ok(value) => value,
            Err(_) => {
                diagnostics.push(diagnostic(line_number, EvaluationError::InvalidHeaderValue));
                continue;
            }
        };
        let value = match value.to_str() {
            Ok(value) => value.to_owned(),
            Err(_) => {
                diagnostics.push(diagnostic(line_number, EvaluationError::HeaderValueNotText));
                continue;
            }
        };
        if let Some(first_line) = first_lines.get(&name).copied() {
            diagnostics.push(diagnostic(
                line_number,
                EvaluationError::DuplicateHeader {
                    name: name.as_str().to_owned(),
                    first_line,
                },
            ));
            continue;
        }
        first_lines.insert(name.clone(), line_number);
        values.insert(name, value);
    }

    ParsedHeaders {
        values,
        diagnostics,
    }
}

fn diagnostic(line: usize, reason: EvaluationError) -> RequestDiagnostic {
    RequestDiagnostic {
        scope: DiagnosticScope::RequestHeaderLine(line),
        reason,
    }
}
