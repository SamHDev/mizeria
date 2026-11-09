//! Feature discovery, loading, and in-place tag filtering.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::ast::{
    self, Feature, FeatureChild, Rule, RuleChild, Scenario, Tag,
};
use crate::loader::{self, LoadError};
use crate::tags::Tags;

/// One loaded feature document ready for the suite to run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedDocument {
    /// Path or URI shown to listeners.
    pub uri: String,
    /// Owned, possibly tag-filtered feature AST.
    pub feature: Feature<'static>,
}

/// Loaded feature documents (owned ASTs).
///
/// Built via [`FeaturesBuilder`]; tag-filter with [`Features::include`] /
/// [`Features::exclude`].
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Features {
    documents: Vec<LoadedDocument>,
    /// Files that failed to parse while building (counted toward run failures).
    parse_failures: usize,
}

impl Features {
    /// Start a builder for feature path configuration.
    pub fn builder() -> FeaturesBuilder {
        FeaturesBuilder::new()
    }

    /// Loaded documents remaining after any tag filters.
    pub fn documents(&self) -> &[LoadedDocument] {
        &self.documents
    }

    /// Number of feature files that failed to parse during [`FeaturesBuilder::build`].
    pub fn parse_failures(&self) -> usize {
        self.parse_failures
    }

    /// Keep only scenarios / Examples that match `tags`. Mutates in place.
    pub fn include(&mut self, tags: Tags) -> &mut Self {
        self.prune(|effective| tags.matches(effective.iter().copied()));
        self
    }

    /// Drop scenarios / Examples that match `tags`. Mutates in place.
    pub fn exclude(&mut self, tags: Tags) -> &mut Self {
        self.prune(|effective| !tags.matches(effective.iter().copied()));
        self
    }

    fn prune(&mut self, keep: impl Fn(&[ &str ]) -> bool) {
        self.documents
            .retain_mut(|doc| prune_feature(&mut doc.feature, &keep));
    }
}

/// Builder for loading [`Features`].
#[derive(Debug, Clone)]
pub struct FeaturesBuilder {
    paths: Vec<PathBuf>,
    recursive: bool,
    include_paths: Vec<PathBuf>,
    exclude_paths: Vec<PathBuf>,
}

impl FeaturesBuilder {
    /// Empty path list with recursive directory walking enabled.
    pub fn new() -> Self {
        Self {
            paths: Vec::new(),
            recursive: true,
            include_paths: Vec::new(),
            exclude_paths: Vec::new(),
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

    /// Keep only discovered feature files whose path matches `path`.
    ///
    /// A path matches when it [`ends_with`](Path::ends_with) the pattern, or when
    /// the pattern appears as a substring of the path string. Multiple calls are
    /// OR'd: a file is kept if it matches **any** include pattern. When no include
    /// patterns are set, every discovered file is eligible (then excludes apply).
    pub fn include_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.include_paths.push(path.into());
        self
    }

    /// Drop discovered feature files whose path matches `path`.
    ///
    /// Matching rules are the same as [`Self::include_path`]. Multiple calls are
    /// OR'd: a file is dropped if it matches **any** exclude pattern.
    pub fn exclude_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.exclude_paths.push(path.into());
        self
    }

    /// Discover, read, and parse feature files into owned ASTs.
    pub fn build(self) -> Result<Features, LoadError> {
        let discovered = loader::discover_feature_paths(&self.paths, self.recursive)?;
        let filtered: Vec<PathBuf> = discovered
            .into_iter()
            .filter(|path| self.path_allowed(path))
            .collect();
        let sources = loader::read_feature_sources(&filtered)?;
        let mut documents = Vec::new();
        let mut parse_failures = 0;

        for source in sources {
            match ast::parse(&source.source) {
                Ok(doc) => {
                    let Some(feature) = doc.feature else {
                        continue;
                    };
                    documents.push(LoadedDocument {
                        uri: source.uri,
                        feature: feature.into_owned(),
                    });
                }
                Err(_) => {
                    parse_failures += 1;
                }
            }
        }

        Ok(Features {
            documents,
            parse_failures,
        })
    }

    fn path_allowed(&self, path: &Path) -> bool {
        let included = self.include_paths.is_empty()
            || self
                .include_paths
                .iter()
                .any(|pattern| path_matches(path, pattern));
        let excluded = self
            .exclude_paths
            .iter()
            .any(|pattern| path_matches(path, pattern));
        included && !excluded
    }
}

fn path_matches(path: &Path, pattern: &Path) -> bool {
    if path.ends_with(pattern) {
        return true;
    }
    let path_s = path.to_string_lossy();
    let pattern_s = pattern.to_string_lossy();
    !pattern_s.is_empty() && path_s.contains(pattern_s.as_ref())
}

impl Default for FeaturesBuilder {
    fn default() -> Self {
        Self::new()
    }
}

fn prune_feature(feature: &mut Feature<'static>, keep: &impl Fn(&[&str]) -> bool) -> bool {
    let feature_tags = tag_names(&feature.tags);
    feature.children.retain_mut(|child| match child {
        FeatureChild::Background(_) => true,
        FeatureChild::Scenario(scenario) => {
            prune_scenario(scenario, &feature_tags, &[], keep)
        }
        FeatureChild::Rule(rule) => prune_rule(rule, &feature_tags, keep),
    });

    let has_runnable = feature.children.iter().any(|child| match child {
        FeatureChild::Scenario(_) | FeatureChild::Rule(_) => true,
        FeatureChild::Background(_) => false,
    });
    has_runnable
}

fn prune_rule(
    rule: &mut Rule<'static>,
    feature_tags: &[&str],
    keep: &impl Fn(&[&str]) -> bool,
) -> bool {
    let rule_tags = tag_names(&rule.tags);
    rule.children.retain_mut(|child| match child {
        RuleChild::Background(_) => true,
        RuleChild::Scenario(scenario) => {
            prune_scenario(scenario, feature_tags, &rule_tags, keep)
        }
    });

    rule.children.iter().any(|child| matches!(child, RuleChild::Scenario(_)))
}

fn prune_scenario(
    scenario: &mut Scenario<'static>,
    feature_tags: &[&str],
    rule_tags: &[&str],
    keep: &impl Fn(&[&str]) -> bool,
) -> bool {
    let scenario_tags = tag_names(&scenario.tags);

    if scenario.examples.is_empty() {
        let effective = merge_tags(feature_tags, rule_tags, &scenario_tags, &[]);
        return keep(&effective);
    }

    scenario.examples.retain_mut(|examples| {
        let examples_tags = tag_names(&examples.tags);
        let effective = merge_tags(feature_tags, rule_tags, &scenario_tags, &examples_tags);
        if !keep(&effective) {
            return false;
        }
        // Whole Examples block shares one tag set; keep all body rows when the block matches.
        true
    });

    !scenario.examples.is_empty()
}

fn tag_names<'a>(tags: &'a [Tag<'static>]) -> Vec<&'a str> {
    tags.iter().map(|t| t.name.as_ref()).collect()
}

fn merge_tags<'a>(
    feature: &[&'a str],
    rule: &[&'a str],
    scenario: &[&'a str],
    examples: &[&'a str],
) -> Vec<&'a str> {
    let mut set = HashSet::new();
    let mut out = Vec::new();
    for name in feature.iter().chain(rule).chain(scenario).chain(examples) {
        if set.insert(*name) {
            out.push(*name);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn write_feature(dir: &std::path::Path, name: &str, body: &str) -> PathBuf {
        fs::create_dir_all(dir).unwrap();
        let path = dir.join(name);
        fs::write(&path, body).unwrap();
        path
    }

    fn scenario_names(features: &Features) -> Vec<String> {
        let mut names = Vec::new();
        for doc in features.documents() {
            for child in &doc.feature.children {
                match child {
                    FeatureChild::Scenario(sc) => names.push(sc.name.to_string()),
                    FeatureChild::Rule(rule) => {
                        for child in &rule.children {
                            if let RuleChild::Scenario(sc) = child {
                                names.push(sc.name.to_string());
                            }
                        }
                    }
                    FeatureChild::Background(_) => {}
                }
            }
        }
        names
    }

    #[test]
    fn include_and_exclude_prune_in_place() {
        let dir = std::env::temp_dir().join(format!(
            "cutecumber-tags-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        let path = write_feature(
            &dir,
            "tags.feature",
            "\
@feature
Feature: Tagged

  @smoke
  Scenario: smoke only
    Given a step

  @wip @smoke
  Scenario: smoke and wip
    Given a step

  @slow
  Scenario: slow only
    Given a step
",
        );

        let mut features = Features::builder().add_path(&path).build().unwrap();
        assert_eq!(scenario_names(&features).len(), 3);

        features.include(Tags::parse("@smoke").unwrap());
        assert_eq!(
            scenario_names(&features),
            vec!["smoke only".to_string(), "smoke and wip".to_string()]
        );

        features.exclude(Tags::parse("@wip").unwrap());
        assert_eq!(scenario_names(&features), vec!["smoke only".to_string()]);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn inherits_feature_and_rule_tags() {
        let dir = std::env::temp_dir().join(format!(
            "cutecumber-inherit-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        let path = write_feature(
            &dir,
            "inherit.feature",
            "\
@epic
Feature: Epic

  @rule-tag
  Rule: R

    Scenario: inherits both
      Given a step
",
        );

        let mut features = Features::builder().add_path(&path).build().unwrap();
        features.include(Tags::parse("@epic and @rule-tag").unwrap());
        assert_eq!(scenario_names(&features), vec!["inherits both".to_string()]);

        let mut features = Features::builder().add_path(&path).build().unwrap();
        features.include(Tags::parse("@missing").unwrap());
        assert!(scenario_names(&features).is_empty());
        assert!(features.documents().is_empty());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn filters_examples_blocks() {
        let dir = std::env::temp_dir().join(format!(
            "cutecumber-ex-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        let path = write_feature(
            &dir,
            "outline.feature",
            "\
Feature: Outline

  Scenario Outline: cases
    Given value <n>

    @keep
    Examples: kept
      | n |
      | 1 |

    @drop
    Examples: dropped
      | n |
      | 2 |
",
        );

        let mut features = Features::builder().add_path(&path).build().unwrap();
        features.include(Tags::parse("@keep").unwrap());
        let FeatureChild::Scenario(sc) = &features.documents()[0].feature.children[0] else {
            panic!("expected scenario");
        };
        assert_eq!(sc.examples.len(), 1);
        assert_eq!(sc.examples[0].name.as_ref(), "kept");

        let _ = fs::remove_dir_all(&dir);
    }

    fn feature_uris(features: &Features) -> Vec<String> {
        features
            .documents()
            .iter()
            .map(|d| d.uri.clone())
            .collect()
    }

    #[test]
    fn include_and_exclude_path_filter_discovered_files() {
        let dir = std::env::temp_dir().join(format!(
            "cutecumber-path-filter-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        let smoke_dir = dir.join("smoke");
        let wip_dir = dir.join("wip");
        write_feature(
            &smoke_dir,
            "a.feature",
            "Feature: Smoke\n  Scenario: s\n    Given a step\n",
        );
        write_feature(
            &wip_dir,
            "b.feature",
            "Feature: Wip\n  Scenario: w\n    Given a step\n",
        );
        write_feature(
            &dir,
            "other.feature",
            "Feature: Other\n  Scenario: o\n    Given a step\n",
        );

        let all = Features::builder().add_path(&dir).build().unwrap();
        assert_eq!(all.documents().len(), 3);

        let included = Features::builder()
            .add_path(&dir)
            .include_path("smoke")
            .build()
            .unwrap();
        assert_eq!(included.documents().len(), 1);
        assert!(
            feature_uris(&included)
                .iter()
                .all(|uri| uri.contains("smoke")),
            "{:?}",
            feature_uris(&included)
        );

        let excluded = Features::builder()
            .add_path(&dir)
            .exclude_path("wip")
            .build()
            .unwrap();
        assert_eq!(excluded.documents().len(), 2);
        assert!(
            feature_uris(&excluded)
                .iter()
                .all(|uri| !uri.contains("wip")),
            "{:?}",
            feature_uris(&excluded)
        );

        let both = Features::builder()
            .add_path(&dir)
            .include_path("smoke")
            .include_path("other.feature")
            .exclude_path("wip")
            .build()
            .unwrap();
        assert_eq!(both.documents().len(), 2);

        let _ = fs::remove_dir_all(&dir);
    }
}
