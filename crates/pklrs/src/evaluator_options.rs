use std::collections::HashMap;

use crate::message::{ProjectOrDependency};
use crate::reader::{ModuleReader, ResourceReader};

/// Options for creating a new evaluator.
#[derive(Default)]
pub struct EvaluatorOptions {
    pub allowed_modules: Option<Vec<String>>,
    pub allowed_resources: Option<Vec<String>>,
    pub module_paths: Option<Vec<String>>,
    pub env: Option<HashMap<String, String>>,
    pub properties: Option<HashMap<String, String>>,
    pub timeout_seconds: Option<i64>,
    pub root_dir: Option<String>,
    pub cache_dir: Option<String>,
    pub output_format: Option<String>,
    pub project: Option<ProjectOrDependency>,
    pub module_readers: Vec<Box<dyn ModuleReader>>,
    pub resource_readers: Vec<Box<dyn ResourceReader>>,
}

impl EvaluatorOptions {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create options with sensible defaults for evaluating local .pkl files.
    pub fn preconfigured() -> Self {
        Self {
            allowed_modules: Some(vec![
                "pkl:".into(),
                "repl:".into(),
                "file:".into(),
                "package:".into(),
                "projectpackage:".into(),
                "https:".into(),
            ]),
            allowed_resources: Some(vec![
                "env:".into(),
                "prop:".into(),
                "package:".into(),
                "projectpackage:".into(),
                "https:".into(),
                "file:".into(),
            ]),
            ..Default::default()
        }
    }

    pub fn allowed_modules(mut self, modules: Vec<String>) -> Self {
        self.allowed_modules = Some(modules);
        self
    }

    pub fn allowed_resources(mut self, resources: Vec<String>) -> Self {
        self.allowed_resources = Some(resources);
        self
    }

    pub fn env(mut self, env: HashMap<String, String>) -> Self {
        self.env = Some(env);
        self
    }

    pub fn properties(mut self, props: HashMap<String, String>) -> Self {
        self.properties = Some(props);
        self
    }

    pub fn timeout_seconds(mut self, seconds: i64) -> Self {
        self.timeout_seconds = Some(seconds);
        self
    }

    pub fn root_dir(mut self, dir: impl Into<String>) -> Self {
        self.root_dir = Some(dir.into());
        self
    }

    pub fn cache_dir(mut self, dir: impl Into<String>) -> Self {
        self.cache_dir = Some(dir.into());
        self
    }

    pub fn output_format(mut self, format: impl Into<String>) -> Self {
        self.output_format = Some(format.into());
        self
    }

    pub fn add_module_reader(mut self, reader: Box<dyn ModuleReader>) -> Self {
        self.module_readers.push(reader);
        self
    }

    pub fn add_resource_reader(mut self, reader: Box<dyn ResourceReader>) -> Self {
        self.resource_readers.push(reader);
        self
    }
}
