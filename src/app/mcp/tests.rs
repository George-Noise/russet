use super::*;

#[test]
fn parse_message_extracts_id_method_and_params() {
    let event = parse_message(serde_json::json!({
        "jsonrpc": "2.0",
        "id": 7,
        "method": "tools/call",
        "params": { "name": "undo", "arguments": {} }
    }));

    assert_eq!(event.id, Some(serde_json::json!(7)));
    assert_eq!(event.method, "tools/call");
    assert_eq!(
        event.params.get("name").and_then(Value::as_str),
        Some("undo")
    );
}

#[test]
fn parse_message_defaults_missing_fields() {
    let event = parse_message(serde_json::json!({ "method": "ping" }));

    assert!(event.id.is_none());
    assert_eq!(event.method, "ping");
    assert_eq!(event.params, Value::Null);
}

#[test]
fn initialize_echoes_a_supported_protocol_version() {
    for version in PROTOCOL_VERSIONS {
        let result = initialize_result(&serde_json::json!({ "protocolVersion": version }));

        assert_eq!(
            result.get("protocolVersion").and_then(Value::as_str),
            Some(version)
        );
    }
}

#[test]
fn initialize_falls_back_to_the_newest_version() {
    let result = initialize_result(&serde_json::json!({ "protocolVersion": "1999-01-01" }));

    assert_eq!(
        result.get("protocolVersion").and_then(Value::as_str),
        Some(PROTOCOL_VERSION_LATEST)
    );
    assert_eq!(
        result
            .get("serverInfo")
            .and_then(|info| info.get("name"))
            .and_then(Value::as_str),
        Some("russet")
    );
}

#[test]
fn envelope_wraps_results_and_errors() {
    let ok: Value = serde_json::from_str(&envelope(
        serde_json::json!(1),
        Ok(serde_json::json!({ "a": 1 })),
    ))
    .expect("valid json");

    assert_eq!(ok.get("id"), Some(&serde_json::json!(1)));
    assert_eq!(
        ok.get("result").and_then(|r| r.get("a")),
        Some(&serde_json::json!(1))
    );

    let err: Value = serde_json::from_str(&envelope(
        serde_json::json!(2),
        Err((-32601, "Method not found".to_string())),
    ))
    .expect("valid json");

    assert_eq!(ok.get("error"), None);
    assert_eq!(
        err.get("error").and_then(|e| e.get("code")),
        Some(&serde_json::json!(-32601))
    );
    assert!(err.get("result").is_none());
}
