use std::io::{Read, Write};

use rmpv::Value;

use crate::error::{Error, Result};
use crate::message::*;

/// Encode an outgoing message as a 2-element MessagePack array: [type_code, body_map].
pub fn encode_message<W: Write>(writer: &mut W, msg: &OutgoingMessage) -> Result<()> {
    let (code, body) = match msg {
        OutgoingMessage::CreateEvaluatorRequest(req) => {
            (MessageCode::CreateEvaluatorRequest as u8, encode_create_evaluator_request(req))
        }
        OutgoingMessage::CloseEvaluator(req) => {
            (MessageCode::CloseEvaluator as u8, encode_close_evaluator(req))
        }
        OutgoingMessage::EvaluateRequest(req) => {
            (MessageCode::EvaluateRequest as u8, encode_evaluate_request(req))
        }
        OutgoingMessage::ReadResourceResponse(resp) => {
            (MessageCode::ReadResourceResponse as u8, encode_read_resource_response(resp))
        }
        OutgoingMessage::ReadModuleResponse(resp) => {
            (MessageCode::ReadModuleResponse as u8, encode_read_module_response(resp))
        }
        OutgoingMessage::ListResourcesResponse(resp) => {
            (MessageCode::ListResourcesResponse as u8, encode_list_resources_response(resp))
        }
        OutgoingMessage::ListModulesResponse(resp) => {
            (MessageCode::ListModulesResponse as u8, encode_list_modules_response(resp))
        }
    };

    let envelope = Value::Array(vec![Value::from(code as u64), body]);
    rmpv::encode::write_value(writer, &envelope)
        .map_err(|e| Error::MsgpackEncode(e.to_string()))?;
    writer.flush()?;
    Ok(())
}

/// Decode an incoming message from a 2-element MessagePack array.
pub fn decode_message<R: Read>(reader: &mut R) -> Result<IncomingMessage> {
    let value = rmpv::decode::read_value(reader)?;
    let arr = value
        .as_array()
        .ok_or_else(|| Error::Decode("expected array".into()))?;
    if arr.len() < 2 {
        return Err(Error::Decode("message array too short".into()));
    }

    let code = arr[0]
        .as_u64()
        .ok_or_else(|| Error::Decode("expected integer type code".into()))? as u8;
    let body = &arr[1];

    match MessageCode::from_u8(code) {
        Some(MessageCode::CreateEvaluatorResponse) => {
            Ok(IncomingMessage::CreateEvaluatorResponse(decode_create_evaluator_response(body)?))
        }
        Some(MessageCode::EvaluateResponse) => {
            Ok(IncomingMessage::EvaluateResponse(decode_evaluate_response(body)?))
        }
        Some(MessageCode::LogMessage) => {
            Ok(IncomingMessage::LogMessage(decode_log_message(body)?))
        }
        Some(MessageCode::ReadResourceRequest) => {
            Ok(IncomingMessage::ReadResourceRequest(decode_read_resource_request(body)?))
        }
        Some(MessageCode::ReadModuleRequest) => {
            Ok(IncomingMessage::ReadModuleRequest(decode_read_module_request(body)?))
        }
        Some(MessageCode::ListResourcesRequest) => {
            Ok(IncomingMessage::ListResourcesRequest(decode_list_resources_request(body)?))
        }
        Some(MessageCode::ListModulesRequest) => {
            Ok(IncomingMessage::ListModulesRequest(decode_list_modules_request(body)?))
        }
        _ => Err(Error::UnexpectedMessageType(code)),
    }
}

// --- Helper functions ---

fn get_str(map: &[(Value, Value)], key: &str) -> Result<String> {
    for (k, v) in map {
        if k.as_str() == Some(key) {
            return v
                .as_str()
                .map(|s| s.to_string())
                .ok_or_else(|| Error::Decode(format!("expected string for key '{key}'")));
        }
    }
    Err(Error::Decode(format!("missing key '{key}'")))
}

fn get_i64(map: &[(Value, Value)], key: &str) -> Result<i64> {
    for (k, v) in map {
        if k.as_str() == Some(key) {
            return v
                .as_i64()
                .ok_or_else(|| Error::Decode(format!("expected integer for key '{key}'")));
        }
    }
    Err(Error::Decode(format!("missing key '{key}'")))
}

fn get_optional_str(map: &[(Value, Value)], key: &str) -> Option<String> {
    for (k, v) in map {
        if k.as_str() == Some(key) {
            if v.is_nil() {
                return None;
            }
            return v.as_str().map(|s| s.to_string());
        }
    }
    None
}

fn get_optional_i64(map: &[(Value, Value)], key: &str) -> Option<i64> {
    for (k, v) in map {
        if k.as_str() == Some(key) {
            if v.is_nil() {
                return None;
            }
            return v.as_i64();
        }
    }
    None
}

fn get_optional_bytes(map: &[(Value, Value)], key: &str) -> Option<Vec<u8>> {
    for (k, v) in map {
        if k.as_str() == Some(key) {
            if v.is_nil() {
                return None;
            }
            return v.as_slice().map(|s| s.to_vec());
        }
    }
    None
}

fn as_map(v: &Value) -> Result<&Vec<(Value, Value)>> {
    v.as_map()
        .ok_or_else(|| Error::Decode("expected map".into()))
}

// --- Encoders ---

fn str_val(s: &str) -> Value {
    Value::String(s.into())
}

fn int_val(i: i64) -> Value {
    Value::from(i)
}

fn map_insert(entries: &mut Vec<(Value, Value)>, key: &str, value: Value) {
    entries.push((str_val(key), value));
}

fn encode_create_evaluator_request(req: &CreateEvaluatorRequest) -> Value {
    let mut entries = Vec::new();
    map_insert(&mut entries, "requestId", int_val(req.request_id));

    if let Some(ref allowed) = req.allowed_modules {
        let arr: Vec<Value> = allowed.iter().map(|s| str_val(s)).collect();
        map_insert(&mut entries, "allowedModules", Value::Array(arr));
    }
    if let Some(ref allowed) = req.allowed_resources {
        let arr: Vec<Value> = allowed.iter().map(|s| str_val(s)).collect();
        map_insert(&mut entries, "allowedResources", Value::Array(arr));
    }
    if let Some(ref readers) = req.client_module_readers {
        let arr: Vec<Value> = readers.iter().map(encode_module_reader_spec).collect();
        map_insert(&mut entries, "clientModuleReaders", Value::Array(arr));
    }
    if let Some(ref readers) = req.client_resource_readers {
        let arr: Vec<Value> = readers.iter().map(encode_resource_reader_spec).collect();
        map_insert(&mut entries, "clientResourceReaders", Value::Array(arr));
    }
    if let Some(ref paths) = req.module_paths {
        let arr: Vec<Value> = paths.iter().map(|s| str_val(s)).collect();
        map_insert(&mut entries, "modulePaths", Value::Array(arr));
    }
    if let Some(ref env) = req.env {
        let m: Vec<(Value, Value)> = env.iter().map(|(k, v)| (str_val(k), str_val(v))).collect();
        map_insert(&mut entries, "env", Value::Map(m));
    }
    if let Some(ref props) = req.properties {
        let m: Vec<(Value, Value)> =
            props.iter().map(|(k, v)| (str_val(k), str_val(v))).collect();
        map_insert(&mut entries, "properties", Value::Map(m));
    }
    if let Some(timeout) = req.timeout_seconds {
        map_insert(&mut entries, "timeoutSeconds", int_val(timeout));
    }
    if let Some(ref root) = req.root_dir {
        map_insert(&mut entries, "rootDir", str_val(root));
    }
    if let Some(ref cache) = req.cache_dir {
        map_insert(&mut entries, "cacheDir", str_val(cache));
    }
    if let Some(ref fmt) = req.output_format {
        map_insert(&mut entries, "outputFormat", str_val(fmt));
    }
    if let Some(ref project) = req.project {
        map_insert(&mut entries, "project", encode_project(project));
    }

    Value::Map(entries)
}

fn encode_module_reader_spec(spec: &ModuleReaderSpec) -> Value {
    let mut entries = Vec::new();
    map_insert(&mut entries, "scheme", str_val(&spec.scheme));
    map_insert(
        &mut entries,
        "hasHierarchicalUris",
        Value::Boolean(spec.has_hierarchical_uris),
    );
    map_insert(&mut entries, "isLocal", Value::Boolean(spec.is_local));
    map_insert(
        &mut entries,
        "isGlobbable",
        Value::Boolean(spec.is_globbable),
    );
    Value::Map(entries)
}

fn encode_resource_reader_spec(spec: &ResourceReaderSpec) -> Value {
    let mut entries = Vec::new();
    map_insert(&mut entries, "scheme", str_val(&spec.scheme));
    map_insert(
        &mut entries,
        "hasHierarchicalUris",
        Value::Boolean(spec.has_hierarchical_uris),
    );
    map_insert(
        &mut entries,
        "isGlobbable",
        Value::Boolean(spec.is_globbable),
    );
    Value::Map(entries)
}

fn encode_project(project: &ProjectOrDependency) -> Value {
    let mut entries = Vec::new();
    if let Some(ref uri) = project.package_uri {
        map_insert(&mut entries, "packageUri", str_val(uri));
    }
    map_insert(&mut entries, "type", str_val(&project.r#type));
    if let Some(ref uri) = project.project_file_uri {
        map_insert(&mut entries, "projectFileUri", str_val(uri));
    }
    if let Some(ref checksums) = project.checksums {
        let m: Vec<(Value, Value)> = checksums
            .iter()
            .map(|(k, v)| (str_val(k), str_val(v)))
            .collect();
        map_insert(&mut entries, "checksums", Value::Map(m));
    }
    if let Some(ref deps) = project.dependencies {
        let m: Vec<(Value, Value)> = deps
            .iter()
            .map(|(k, v)| (str_val(k), encode_project(v)))
            .collect();
        map_insert(&mut entries, "dependencies", Value::Map(m));
    }
    Value::Map(entries)
}

fn encode_close_evaluator(req: &CloseEvaluator) -> Value {
    let mut entries = Vec::new();
    map_insert(&mut entries, "evaluatorId", int_val(req.evaluator_id));
    Value::Map(entries)
}

fn encode_evaluate_request(req: &EvaluateRequest) -> Value {
    let mut entries = Vec::new();
    map_insert(&mut entries, "requestId", int_val(req.request_id));
    map_insert(&mut entries, "evaluatorId", int_val(req.evaluator_id));
    map_insert(&mut entries, "moduleUri", str_val(&req.module_uri));
    if let Some(ref text) = req.module_text {
        map_insert(&mut entries, "moduleText", str_val(text));
    }
    if let Some(ref expr) = req.expr {
        map_insert(&mut entries, "expr", str_val(expr));
    }
    Value::Map(entries)
}

fn encode_read_resource_response(resp: &ReadResourceResponse) -> Value {
    let mut entries = Vec::new();
    map_insert(&mut entries, "requestId", int_val(resp.request_id));
    map_insert(&mut entries, "evaluatorId", int_val(resp.evaluator_id));
    if let Some(ref contents) = resp.contents {
        map_insert(
            &mut entries,
            "contents",
            Value::Binary(contents.clone()),
        );
    }
    if let Some(ref error) = resp.error {
        map_insert(&mut entries, "error", str_val(error));
    }
    Value::Map(entries)
}

fn encode_read_module_response(resp: &ReadModuleResponse) -> Value {
    let mut entries = Vec::new();
    map_insert(&mut entries, "requestId", int_val(resp.request_id));
    map_insert(&mut entries, "evaluatorId", int_val(resp.evaluator_id));
    if let Some(ref contents) = resp.contents {
        map_insert(&mut entries, "contents", str_val(contents));
    }
    if let Some(ref error) = resp.error {
        map_insert(&mut entries, "error", str_val(error));
    }
    Value::Map(entries)
}

fn encode_path_elements(elements: &[PathElement]) -> Value {
    let arr: Vec<Value> = elements
        .iter()
        .map(|el| {
            let mut entries = Vec::new();
            map_insert(&mut entries, "name", str_val(&el.name));
            map_insert(&mut entries, "isDirectory", Value::Boolean(el.is_directory));
            Value::Map(entries)
        })
        .collect();
    Value::Array(arr)
}

fn encode_list_resources_response(resp: &ListResourcesResponse) -> Value {
    let mut entries = Vec::new();
    map_insert(&mut entries, "requestId", int_val(resp.request_id));
    map_insert(&mut entries, "evaluatorId", int_val(resp.evaluator_id));
    if let Some(ref elements) = resp.path_elements {
        map_insert(&mut entries, "pathElements", encode_path_elements(elements));
    }
    if let Some(ref error) = resp.error {
        map_insert(&mut entries, "error", str_val(error));
    }
    Value::Map(entries)
}

fn encode_list_modules_response(resp: &ListModulesResponse) -> Value {
    let mut entries = Vec::new();
    map_insert(&mut entries, "requestId", int_val(resp.request_id));
    map_insert(&mut entries, "evaluatorId", int_val(resp.evaluator_id));
    if let Some(ref elements) = resp.path_elements {
        map_insert(&mut entries, "pathElements", encode_path_elements(elements));
    }
    if let Some(ref error) = resp.error {
        map_insert(&mut entries, "error", str_val(error));
    }
    Value::Map(entries)
}

// --- Decoders ---

fn decode_create_evaluator_response(body: &Value) -> Result<CreateEvaluatorResponse> {
    let map = as_map(body)?;
    Ok(CreateEvaluatorResponse {
        request_id: get_i64(map, "requestId")?,
        evaluator_id: get_optional_i64(map, "evaluatorId"),
        error: get_optional_str(map, "error"),
    })
}

fn decode_evaluate_response(body: &Value) -> Result<EvaluateResponse> {
    let map = as_map(body)?;
    Ok(EvaluateResponse {
        request_id: get_i64(map, "requestId")?,
        evaluator_id: get_i64(map, "evaluatorId")?,
        result: get_optional_bytes(map, "result"),
        error: get_optional_str(map, "error"),
    })
}

fn decode_log_message(body: &Value) -> Result<LogMessage> {
    let map = as_map(body)?;
    Ok(LogMessage {
        evaluator_id: get_i64(map, "evaluatorId")?,
        level: get_i64(map, "level")?,
        message: get_str(map, "message")?,
        frame_uri: get_str(map, "frameUri")?,
    })
}

fn decode_read_resource_request(body: &Value) -> Result<ReadResourceRequest> {
    let map = as_map(body)?;
    Ok(ReadResourceRequest {
        request_id: get_i64(map, "requestId")?,
        evaluator_id: get_i64(map, "evaluatorId")?,
        uri: get_str(map, "uri")?,
    })
}

fn decode_read_module_request(body: &Value) -> Result<ReadModuleRequest> {
    let map = as_map(body)?;
    Ok(ReadModuleRequest {
        request_id: get_i64(map, "requestId")?,
        evaluator_id: get_i64(map, "evaluatorId")?,
        uri: get_str(map, "uri")?,
    })
}

fn decode_list_resources_request(body: &Value) -> Result<ListResourcesRequest> {
    let map = as_map(body)?;
    Ok(ListResourcesRequest {
        request_id: get_i64(map, "requestId")?,
        evaluator_id: get_i64(map, "evaluatorId")?,
        uri: get_str(map, "uri")?,
    })
}

fn decode_list_modules_request(body: &Value) -> Result<ListModulesRequest> {
    let map = as_map(body)?;
    Ok(ListModulesRequest {
        request_id: get_i64(map, "requestId")?,
        evaluator_id: get_i64(map, "evaluatorId")?,
        uri: get_str(map, "uri")?,
    })
}
