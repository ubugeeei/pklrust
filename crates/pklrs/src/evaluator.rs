use std::sync::atomic::{AtomicI64, Ordering};

use serde::de::DeserializeOwned;

use crate::de::from_pkl_value;
use crate::decoder::decode_pkl_binary;
use crate::error::{Error, Result};
use crate::evaluator_options::EvaluatorOptions;
use crate::message::*;
use crate::module_source::ModuleSource;
use crate::process::PklProcess;
use crate::reader::{ModuleReader, ResourceReader};
use crate::value::PklValue;

static REQUEST_ID_COUNTER: AtomicI64 = AtomicI64::new(1);

fn next_request_id() -> i64 {
    REQUEST_ID_COUNTER.fetch_add(1, Ordering::Relaxed)
}

/// Manages pkl server processes and evaluator instances.
pub struct EvaluatorManager {
    process: PklProcess,
}

impl EvaluatorManager {
    /// Create a new EvaluatorManager, starting a pkl server process.
    pub fn new() -> Result<Self> {
        Self::with_command("pkl")
    }

    /// Create a new EvaluatorManager with a custom pkl command.
    pub fn with_command(pkl_command: &str) -> Result<Self> {
        let process = PklProcess::start_with_command(pkl_command)?;
        Ok(Self { process })
    }

    /// Create a new evaluator with the given options.
    pub fn new_evaluator(&mut self, opts: EvaluatorOptions) -> Result<Evaluator> {
        let request_id = next_request_id();

        let client_module_readers: Option<Vec<ModuleReaderSpec>> = if opts.module_readers.is_empty()
        {
            None
        } else {
            Some(
                opts.module_readers
                    .iter()
                    .map(|r| ModuleReaderSpec {
                        scheme: r.scheme().to_string(),
                        has_hierarchical_uris: r.has_hierarchical_uris(),
                        is_local: r.is_local(),
                        is_globbable: r.is_globbable(),
                    })
                    .collect(),
            )
        };

        let client_resource_readers: Option<Vec<ResourceReaderSpec>> =
            if opts.resource_readers.is_empty() {
                None
            } else {
                Some(
                    opts.resource_readers
                        .iter()
                        .map(|r| ResourceReaderSpec {
                            scheme: r.scheme().to_string(),
                            has_hierarchical_uris: r.has_hierarchical_uris(),
                            is_globbable: r.is_globbable(),
                        })
                        .collect(),
                )
            };

        let req = CreateEvaluatorRequest {
            request_id,
            allowed_modules: opts.allowed_modules,
            allowed_resources: opts.allowed_resources,
            client_module_readers,
            client_resource_readers,
            module_paths: opts.module_paths,
            env: opts.env,
            properties: opts.properties,
            timeout_seconds: opts.timeout_seconds,
            root_dir: opts.root_dir,
            cache_dir: opts.cache_dir,
            output_format: opts.output_format,
            project: opts.project,
        };

        self.process
            .send(&OutgoingMessage::CreateEvaluatorRequest(req))?;

        // Read response
        let resp = self.process.recv()?;
        match resp {
            IncomingMessage::CreateEvaluatorResponse(resp) => {
                if let Some(error) = resp.error {
                    return Err(Error::PklServer(error));
                }
                let evaluator_id = resp
                    .evaluator_id
                    .ok_or_else(|| Error::PklServer("no evaluator_id in response".into()))?;
                Ok(Evaluator {
                    evaluator_id,
                    module_readers: opts.module_readers,
                    resource_readers: opts.resource_readers,
                })
            }
            _ => Err(Error::UnexpectedMessageType(0)),
        }
    }

    /// Evaluate a module and return the raw PklValue.
    pub fn evaluate_module(
        &mut self,
        evaluator: &Evaluator,
        source: ModuleSource,
    ) -> Result<PklValue> {
        self.evaluate_expression(evaluator, source, None)
    }

    /// Evaluate a module and deserialize the result into a Rust type.
    pub fn evaluate_module_typed<T: DeserializeOwned>(
        &mut self,
        evaluator: &Evaluator,
        source: ModuleSource,
    ) -> Result<T> {
        let value = self.evaluate_module(evaluator, source)?;
        from_pkl_value(&value)
    }

    /// Evaluate an expression within a module.
    pub fn evaluate_expression(
        &mut self,
        evaluator: &Evaluator,
        source: ModuleSource,
        expr: Option<&str>,
    ) -> Result<PklValue> {
        let request_id = next_request_id();

        let req = EvaluateRequest {
            request_id,
            evaluator_id: evaluator.evaluator_id,
            module_uri: source.module_uri(),
            module_text: source.module_text().map(|s| s.to_string()),
            expr: expr.map(|s| s.to_string()),
        };

        self.process
            .send(&OutgoingMessage::EvaluateRequest(req))?;

        // Message loop: handle log messages and reader requests until we get the response
        loop {
            let msg = self.process.recv()?;
            match msg {
                IncomingMessage::EvaluateResponse(resp) => {
                    if let Some(error) = resp.error {
                        return Err(Error::Evaluation(error));
                    }
                    let result_bytes = resp
                        .result
                        .ok_or_else(|| Error::Evaluation("no result in response".into()))?;
                    return decode_pkl_binary(&result_bytes);
                }
                IncomingMessage::LogMessage(log) => {
                    let level = if log.level == 0 { "TRACE" } else { "WARN" };
                    eprintln!("[pkl {level}] {}: {}", log.frame_uri, log.message);
                }
                IncomingMessage::ReadResourceRequest(req) => {
                    self.handle_read_resource(&evaluator, &req)?;
                }
                IncomingMessage::ReadModuleRequest(req) => {
                    self.handle_read_module(&evaluator, &req)?;
                }
                IncomingMessage::ListResourcesRequest(req) => {
                    self.handle_list_resources(&evaluator, &req)?;
                }
                IncomingMessage::ListModulesRequest(req) => {
                    self.handle_list_modules(&evaluator, &req)?;
                }
                _ => {
                    return Err(Error::UnexpectedMessageType(0));
                }
            }
        }
    }

    /// Close an evaluator.
    pub fn close_evaluator(&mut self, evaluator: &Evaluator) -> Result<()> {
        self.process
            .send(&OutgoingMessage::CloseEvaluator(CloseEvaluator {
                evaluator_id: evaluator.evaluator_id,
            }))
    }

    fn handle_read_resource(
        &mut self,
        evaluator: &Evaluator,
        req: &ReadResourceRequest,
    ) -> Result<()> {
        let scheme = req
            .uri
            .split(':')
            .next()
            .unwrap_or("");

        let result = evaluator
            .resource_readers
            .iter()
            .find(|r| r.scheme() == scheme)
            .map(|r| r.read(&req.uri));

        let resp = match result {
            Some(Ok(contents)) => ReadResourceResponse {
                request_id: req.request_id,
                evaluator_id: req.evaluator_id,
                contents: Some(contents),
                error: None,
            },
            Some(Err(e)) => ReadResourceResponse {
                request_id: req.request_id,
                evaluator_id: req.evaluator_id,
                contents: None,
                error: Some(e),
            },
            None => ReadResourceResponse {
                request_id: req.request_id,
                evaluator_id: req.evaluator_id,
                contents: None,
                error: Some(format!("no reader for scheme: {scheme}")),
            },
        };

        self.process
            .send(&OutgoingMessage::ReadResourceResponse(resp))
    }

    fn handle_read_module(
        &mut self,
        evaluator: &Evaluator,
        req: &ReadModuleRequest,
    ) -> Result<()> {
        let scheme = req.uri.split(':').next().unwrap_or("");

        let result = evaluator
            .module_readers
            .iter()
            .find(|r| r.scheme() == scheme)
            .map(|r| r.read(&req.uri));

        let resp = match result {
            Some(Ok(contents)) => ReadModuleResponse {
                request_id: req.request_id,
                evaluator_id: req.evaluator_id,
                contents: Some(contents),
                error: None,
            },
            Some(Err(e)) => ReadModuleResponse {
                request_id: req.request_id,
                evaluator_id: req.evaluator_id,
                contents: None,
                error: Some(e),
            },
            None => ReadModuleResponse {
                request_id: req.request_id,
                evaluator_id: req.evaluator_id,
                contents: None,
                error: Some(format!("no reader for scheme: {scheme}")),
            },
        };

        self.process
            .send(&OutgoingMessage::ReadModuleResponse(resp))
    }

    fn handle_list_resources(
        &mut self,
        evaluator: &Evaluator,
        req: &ListResourcesRequest,
    ) -> Result<()> {
        let scheme = req.uri.split(':').next().unwrap_or("");

        let result = evaluator
            .resource_readers
            .iter()
            .find(|r| r.scheme() == scheme)
            .map(|r| r.list(&req.uri));

        let resp = match result {
            Some(Ok(elements)) => ListResourcesResponse {
                request_id: req.request_id,
                evaluator_id: req.evaluator_id,
                path_elements: Some(elements),
                error: None,
            },
            Some(Err(e)) => ListResourcesResponse {
                request_id: req.request_id,
                evaluator_id: req.evaluator_id,
                path_elements: None,
                error: Some(e),
            },
            None => ListResourcesResponse {
                request_id: req.request_id,
                evaluator_id: req.evaluator_id,
                path_elements: None,
                error: Some(format!("no reader for scheme: {scheme}")),
            },
        };

        self.process
            .send(&OutgoingMessage::ListResourcesResponse(resp))
    }

    fn handle_list_modules(
        &mut self,
        evaluator: &Evaluator,
        req: &ListModulesRequest,
    ) -> Result<()> {
        let scheme = req.uri.split(':').next().unwrap_or("");

        let result = evaluator
            .module_readers
            .iter()
            .find(|r| r.scheme() == scheme)
            .map(|r| r.list(&req.uri));

        let resp = match result {
            Some(Ok(elements)) => ListModulesResponse {
                request_id: req.request_id,
                evaluator_id: req.evaluator_id,
                path_elements: Some(elements),
                error: None,
            },
            Some(Err(e)) => ListModulesResponse {
                request_id: req.request_id,
                evaluator_id: req.evaluator_id,
                path_elements: None,
                error: Some(e),
            },
            None => ListModulesResponse {
                request_id: req.request_id,
                evaluator_id: req.evaluator_id,
                path_elements: None,
                error: Some(format!("no reader for scheme: {scheme}")),
            },
        };

        self.process
            .send(&OutgoingMessage::ListModulesResponse(resp))
    }
}

/// A handle to a pkl evaluator instance.
pub struct Evaluator {
    pub(crate) evaluator_id: i64,
    pub(crate) module_readers: Vec<Box<dyn ModuleReader>>,
    pub(crate) resource_readers: Vec<Box<dyn ResourceReader>>,
}

impl Evaluator {
    /// Get the evaluator ID.
    pub fn id(&self) -> i64 {
        self.evaluator_id
    }
}
