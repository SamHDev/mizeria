//! Feature discovery configuration (paths only; loading comes later).

use std::path::PathBuf;

/// Configured set of feature paths ready to be loaded by a suite run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Features {
    paths: Vec<PathBuf>,
    recursive: bool,
}

impl Features {
    /// Start a builder for feature path configuration.
    pub fn new() -> FeaturesBuilder {
        FeaturesBuilder::new()
    }

    /// Paths registered via [`FeaturesBuilder::add_path`].
    pub fn paths(&self) -> &[PathBuf] {
        &self.paths
    }

    /// Whether directory paths should be walked recursively.
    pub fn recursive(&self) -> bool {
        self.recursive
    }
}

impl Default for Features {
    fn default() -> Self {
        FeaturesBuilder::new().build()
    }
}

/// Builder for [`Features`].
#[derive(Debug, Clone)]
pub struct FeaturesBuilder {
    paths: Vec<PathBuf>,
    recursive: bool,
}

impl FeaturesBuilder {
    pub fn new() -> Self {
        Self {
            paths: Vec::new(),
            recursive: true,
        }
    }

    /// Add a file or directory to search for `.feature` files.
    pub fn add_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.paths.push(path.into());
        self
    }

    /// Control recursive directory walking (default: `true`).
    pub fn recursive(mut self, recursive: bool) -> Self {
        self.recursive = recursive;
        self
    }

    /// Finish configuration. Does not touch the filesystem.
    pub fn build(self) -> Features {
        Features {
            paths: self.paths,
            recursive: self.recursive,
        }
    }
}

impl Default for FeaturesBuilder {
    fn default() -> Self {
        Self::new()
    }
}
