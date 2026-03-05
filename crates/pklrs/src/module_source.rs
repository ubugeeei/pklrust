use std::path::Path;

/// Represents the source of a Pkl module to evaluate.
#[derive(Debug, Clone)]
pub enum ModuleSource {
    /// A file path to a .pkl file.
    File(String),
    /// Inline Pkl source text with an optional name URI.
    Text { uri: String, text: String },
    /// A URI pointing to a module.
    Uri(String),
}

impl ModuleSource {
    /// Create a ModuleSource from a file path.
    pub fn file(path: impl AsRef<Path>) -> Self {
        let abs = std::fs::canonicalize(path.as_ref())
            .unwrap_or_else(|_| path.as_ref().to_path_buf());
        Self::File(abs.display().to_string())
    }

    /// Create a ModuleSource from inline text.
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text {
            uri: "repl:text".into(),
            text: text.into(),
        }
    }

    /// Create a ModuleSource from inline text with a custom name.
    pub fn text_with_uri(uri: impl Into<String>, text: impl Into<String>) -> Self {
        Self::Text {
            uri: uri.into(),
            text: text.into(),
        }
    }

    /// Create a ModuleSource from a URI.
    pub fn uri(uri: impl Into<String>) -> Self {
        Self::Uri(uri.into())
    }

    /// Get the module URI for the request.
    pub fn module_uri(&self) -> String {
        match self {
            Self::File(path) => format!("file://{path}"),
            Self::Text { uri, .. } => uri.clone(),
            Self::Uri(uri) => uri.clone(),
        }
    }

    /// Get the module text, if this is an inline source.
    pub fn module_text(&self) -> Option<&str> {
        match self {
            Self::Text { text, .. } => Some(text),
            _ => None,
        }
    }
}
