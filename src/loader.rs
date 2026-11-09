//! Discover and read `.feature` sources from [`Features`](crate::Features) paths.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::features::Features;

/// One feature file on disk, ready to parse.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeatureSource {
    pub uri: String,
    pub source: String,
}

/// Errors while discovering or reading feature files.
#[derive(Debug)]
pub enum LoadError {
    Io { path: PathBuf, source: io::Error },
    NotFound { path: PathBuf },
    NotFeature { path: PathBuf },
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

/// Collect feature file contents for a suite run.
pub fn load_features(features: &Features) -> Result<Vec<FeatureSource>, LoadError> {
    let mut paths = Vec::new();
    for root in features.paths() {
        collect_paths(root, features.recursive(), &mut paths)?;
    }
    paths.sort();
    paths.dedup();

    let mut out = Vec::with_capacity(paths.len());
    for path in paths {
        let source = fs::read_to_string(&path).map_err(|source| LoadError::Io {
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
