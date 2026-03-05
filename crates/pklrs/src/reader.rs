use crate::message::{PathElement};

/// A custom module reader that can resolve modules by URI scheme.
pub trait ModuleReader: Send + Sync {
    /// The URI scheme this reader handles (e.g., "myscheme").
    fn scheme(&self) -> &str;

    /// Whether URIs of this scheme are hierarchical.
    fn has_hierarchical_uris(&self) -> bool;

    /// Whether this scheme refers to local resources.
    fn is_local(&self) -> bool;

    /// Whether this reader supports globbing.
    fn is_globbable(&self) -> bool;

    /// Read the module source at the given URI.
    fn read(&self, uri: &str) -> Result<String, String>;

    /// List modules at the given URI (for globbing support).
    fn list(&self, uri: &str) -> Result<Vec<PathElement>, String> {
        let _ = uri;
        Err("listing not supported".into())
    }
}

/// A custom resource reader that can resolve resources by URI scheme.
pub trait ResourceReader: Send + Sync {
    /// The URI scheme this reader handles.
    fn scheme(&self) -> &str;

    /// Whether URIs of this scheme are hierarchical.
    fn has_hierarchical_uris(&self) -> bool;

    /// Whether this reader supports globbing.
    fn is_globbable(&self) -> bool;

    /// Read the resource at the given URI.
    fn read(&self, uri: &str) -> Result<Vec<u8>, String>;

    /// List resources at the given URI (for globbing support).
    fn list(&self, uri: &str) -> Result<Vec<PathElement>, String> {
        let _ = uri;
        Err("listing not supported".into())
    }
}
