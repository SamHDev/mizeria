//! Discover and read `.feature` sources from configured paths.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// One feature file on disk, ready to parse.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeatureSource {
    /// Path or URI shown to listeners (usually the filesystem path).
    pub uri: String,
    /// Raw Gherkin source text.
    pub source: String,
}

/// Errors while discovering or reading feature files.
#[derive(Debug)]
pub enum LoadError {
    /// Filesystem I/O failed while reading `path`.
    Io {
        /// Path that caused the error.
        path: PathBuf,
        /// Underlying I/O error.
        source: io::Error,
    },
    /// Configured path does not exist.
    NotFound {
        /// Missing path.
        path: PathBuf,
    },
    /// Path exists but is neither a `.feature` file nor a directory.
    NotFeature {
        /// Rejected path.
        path: PathBuf,
    },
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io { path, source } => write!(f, "failed to read {}: {source}", path.display()),
            Self::NotFound { path } => write!(f, "feature path not found: {}", path.display()),
            Self::NotFeature { path } => {
                write!(
                    f,
                    "path is not a .feature file or directory: {}",
                    path.display()
                )
            }
        }
    }
}

impl std::error::Error for LoadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::NotFound { .. } | Self::NotFeature { .. } => None,
        }
    }
}

/// Discover `.feature` paths under the given roots.
///
/// Directories are walked according to `recursive`. Paths are sorted and
/// deduplicated.
pub fn discover_feature_paths(
    paths: &[PathBuf],
    recursive: bool,
) -> Result<Vec<PathBuf>, LoadError> {
    let mut discovered = Vec::new();
    for root in paths {
        collect_paths(root, recursive, &mut discovered)?;
    }
    discovered.sort();
    discovered.dedup();
    Ok(discovered)
}

/// Collect feature file contents for the given roots.
///
/// Directories are walked according to `recursive`. Paths are sorted and
/// deduplicated before reading.
pub fn load_feature_sources(
    paths: &[PathBuf],
    recursive: bool,
) -> Result<Vec<FeatureSource>, LoadError> {
    let discovered = discover_feature_paths(paths, recursive)?;
    read_feature_sources(&discovered)
}

/// Read the given feature file paths into [`FeatureSource`] values.
pub fn read_feature_sources(paths: &[PathBuf]) -> Result<Vec<FeatureSource>, LoadError> {
    let mut out = Vec::with_capacity(paths.len());
    for path in paths {
        let source = fs::read_to_string(path).map_err(|source| LoadError::Io {
            path: path.clone(),
            source,
        })?;
        out.push(FeatureSource {
            uri: path.to_string_lossy().into_owned(),
            source,
        });
    }
    Ok(out)
}

fn collect_paths(path: &Path, recursive: bool, out: &mut Vec<PathBuf>) -> Result<(), LoadError> {
    if !path.exists() {
        return Err(LoadError::NotFound {
            path: path.to_path_buf(),
        });
    }

    if path.is_file() {
        if is_feature_file(path) {
            out.push(path.to_path_buf());
            Ok(())
        } else {
            Err(LoadError::NotFeature {
                path: path.to_path_buf(),
            })
        }
    } else if path.is_dir() {
        collect_dir(path, recursive, out)
    } else {
        Err(LoadError::NotFeature {
            path: path.to_path_buf(),
        })
    }
}

fn collect_dir(dir: &Path, recursive: bool, out: &mut Vec<PathBuf>) -> Result<(), LoadError> {
    let entries = fs::read_dir(dir).map_err(|source| LoadError::Io {
        path: dir.to_path_buf(),
        source,
    })?;

    for entry in entries {
        let entry = entry.map_err(|source| LoadError::Io {
            path: dir.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        if path.is_dir() {
            if recursive {
                collect_dir(&path, true, out)?;
            }
        } else if is_feature_file(&path) {
            out.push(path);
        }
    }
    Ok(())
}

fn is_feature_file(path: &Path) -> bool {
    path.extension().and_then(|e| e.to_str()) == Some("feature")
}
