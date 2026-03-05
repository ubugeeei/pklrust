use std::collections::HashMap;

/// Message type codes for the pkl server IPC protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MessageCode {
    CreateEvaluatorRequest = 0x20,
    CreateEvaluatorResponse = 0x21,
    CloseEvaluator = 0x22,
    EvaluateRequest = 0x23,
    EvaluateResponse = 0x24,
    LogMessage = 0x25,
    ReadResourceRequest = 0x26,
    ReadResourceResponse = 0x27,
    ReadModuleRequest = 0x28,
    ReadModuleResponse = 0x29,
    ListResourcesRequest = 0x2A,
    ListResourcesResponse = 0x2B,
    ListModulesRequest = 0x2C,
    ListModulesResponse = 0x2D,
}

impl MessageCode {
    pub fn from_u8(code: u8) -> Option<Self> {
        match code {
            0x20 => Some(Self::CreateEvaluatorRequest),
            0x21 => Some(Self::CreateEvaluatorResponse),
            0x22 => Some(Self::CloseEvaluator),
            0x23 => Some(Self::EvaluateRequest),
            0x24 => Some(Self::EvaluateResponse),
            0x25 => Some(Self::LogMessage),
            0x26 => Some(Self::ReadResourceRequest),
            0x27 => Some(Self::ReadResourceResponse),
            0x28 => Some(Self::ReadModuleRequest),
            0x29 => Some(Self::ReadModuleResponse),
            0x2A => Some(Self::ListResourcesRequest),
            0x2B => Some(Self::ListResourcesResponse),
            0x2C => Some(Self::ListModulesRequest),
            0x2D => Some(Self::ListModulesResponse),
            _ => None,
        }
    }
}

/// Incoming messages from the pkl server.
#[derive(Debug, Clone)]
pub enum IncomingMessage {
    CreateEvaluatorResponse(CreateEvaluatorResponse),
    EvaluateResponse(EvaluateResponse),
    LogMessage(LogMessage),
    ReadResourceRequest(ReadResourceRequest),
    ReadModuleRequest(ReadModuleRequest),
    ListResourcesRequest(ListResourcesRequest),
    ListModulesRequest(ListModulesRequest),
}

/// Outgoing messages to the pkl server.
#[derive(Debug, Clone)]
pub enum OutgoingMessage {
    CreateEvaluatorRequest(CreateEvaluatorRequest),
    CloseEvaluator(CloseEvaluator),
    EvaluateRequest(EvaluateRequest),
    ReadResourceResponse(ReadResourceResponse),
    ReadModuleResponse(ReadModuleResponse),
    ListResourcesResponse(ListResourcesResponse),
    ListModulesResponse(ListModulesResponse),
}

// --- Request/Response types ---

#[derive(Debug, Clone)]
pub struct ModuleReaderSpec {
    pub scheme: String,
    pub has_hierarchical_uris: bool,
    pub is_local: bool,
    pub is_globbable: bool,
}

#[derive(Debug, Clone)]
pub struct ResourceReaderSpec {
    pub scheme: String,
    pub has_hierarchical_uris: bool,
    pub is_globbable: bool,
}

#[derive(Debug, Clone)]
pub struct ProjectOrDependency {
    pub package_uri: Option<String>,
    pub r#type: String,
    pub project_file_uri: Option<String>,
    pub checksums: Option<HashMap<String, String>>,
    pub dependencies: Option<HashMap<String, ProjectOrDependency>>,
}

#[derive(Debug, Clone)]
pub struct CreateEvaluatorRequest {
    pub request_id: i64,
    pub allowed_modules: Option<Vec<String>>,
    pub allowed_resources: Option<Vec<String>>,
    pub client_module_readers: Option<Vec<ModuleReaderSpec>>,
    pub client_resource_readers: Option<Vec<ResourceReaderSpec>>,
    pub module_paths: Option<Vec<String>>,
    pub env: Option<HashMap<String, String>>,
    pub properties: Option<HashMap<String, String>>,
    pub timeout_seconds: Option<i64>,
    pub root_dir: Option<String>,
    pub cache_dir: Option<String>,
    pub output_format: Option<String>,
    pub project: Option<ProjectOrDependency>,
}

#[derive(Debug, Clone)]
pub struct CreateEvaluatorResponse {
    pub request_id: i64,
    pub evaluator_id: Option<i64>,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CloseEvaluator {
    pub evaluator_id: i64,
}

#[derive(Debug, Clone)]
pub struct EvaluateRequest {
    pub request_id: i64,
    pub evaluator_id: i64,
    pub module_uri: String,
    pub module_text: Option<String>,
    pub expr: Option<String>,
}

#[derive(Debug, Clone)]
pub struct EvaluateResponse {
    pub request_id: i64,
    pub evaluator_id: i64,
    pub result: Option<Vec<u8>>,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LogMessage {
    pub evaluator_id: i64,
    pub level: i64,
    pub message: String,
    pub frame_uri: String,
}

#[derive(Debug, Clone)]
pub struct ReadResourceRequest {
    pub request_id: i64,
    pub evaluator_id: i64,
    pub uri: String,
}

#[derive(Debug, Clone)]
pub struct ReadResourceResponse {
    pub request_id: i64,
    pub evaluator_id: i64,
    pub contents: Option<Vec<u8>>,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ReadModuleRequest {
    pub request_id: i64,
    pub evaluator_id: i64,
    pub uri: String,
}

#[derive(Debug, Clone)]
pub struct ReadModuleResponse {
    pub request_id: i64,
    pub evaluator_id: i64,
    pub contents: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PathElement {
    pub name: String,
    pub is_directory: bool,
}

#[derive(Debug, Clone)]
pub struct ListResourcesRequest {
    pub request_id: i64,
    pub evaluator_id: i64,
    pub uri: String,
}

#[derive(Debug, Clone)]
pub struct ListResourcesResponse {
    pub request_id: i64,
    pub evaluator_id: i64,
    pub path_elements: Option<Vec<PathElement>>,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ListModulesRequest {
    pub request_id: i64,
    pub evaluator_id: i64,
    pub uri: String,
}

#[derive(Debug, Clone)]
pub struct ListModulesResponse {
    pub request_id: i64,
    pub evaluator_id: i64,
    pub path_elements: Option<Vec<PathElement>>,
    pub error: Option<String>,
}
