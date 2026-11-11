//! Scenario job collection and outline expansion.

use std::borrow::Cow;
use std::collections::HashSet;

use crate::ast::{Background, FeatureChild, KeywordType, Location, RuleChild, Scenario, Step, Tag};
use crate::features::Features;
use crate::listener::{FeatureEvent, RuleEvent, ScenarioEvent};

/// One runnable scenario (or outline example) with steps already expanded.
pub(crate) struct ScenarioJob<'a> {
    pub feature_uri: &'a str,
    pub feature_name: &'a str,
    pub feature_keyword: &'a str,
    pub feature_location: Location,
    pub feature_key: usize,
    pub rule: Option<RuleMeta<'a>>,
    pub rule_key: Option<usize>,
    pub scenario_name: &'a str,
    pub scenario_keyword: &'a str,
    pub scenario_location: Location,
    pub serial: bool,
    pub steps: Vec<RunnableStep<'a>>,
}

#[derive(Clone, Copy)]
pub(crate) struct RuleMeta<'a> {
    pub name: &'a str,
    pub keyword: &'a str,
    pub location: Location,
}

impl<'a> ScenarioJob<'a> {
    pub fn feature_event(&self) -> FeatureEvent<'a> {
        FeatureEvent {
            uri: self.feature_uri,
            name: self.feature_name,
            keyword: self.feature_keyword,
            location: self.feature_location,
        }
    }

    pub fn rule_event(&self) -> Option<RuleEvent<'a>> {
        self.rule.map(|r| RuleEvent {
            name: r.name,
            keyword: r.keyword,
            location: r.location,
        })
    }

    pub fn scenario_event(&self) -> ScenarioEvent<'a> {
        ScenarioEvent {
            name: self.scenario_name,
            keyword: self.scenario_keyword,
            location: self.scenario_location,
        }
    }
}

#[derive(Debug)]
pub(crate) struct RunnableStep<'a> {
    pub keyword: Cow<'a, str>,
    pub keyword_type: KeywordType,
    pub text: Cow<'a, str>,
    pub location: Location,
    pub doc_string: Option<Cow<'a, str>>,
    pub table: Option<Vec<Vec<String>>>,
}

#[derive(Debug, Default)]
struct ExampleRow {
    /// `(needle, value)` where needle is already `"<key>"`.
    values: Vec<(String, String)>,
}

impl ExampleRow {
    fn needs_substitution(&self) -> bool {
        !self.values.is_empty()
    }

    fn substitute<'a>(&'a self, text: &'a str) -> Cow<'a, str> {
        if self.values.is_empty() || !text.contains('<') {
            return Cow::Borrowed(text);
        }
        let mut out = text.to_owned();
        for (needle, value) in &self.values {
            if out.contains(needle.as_str()) {
                out = out.replace(needle.as_str(), value);
            }
        }
        Cow::Owned(out)
    }
}

struct ScenarioJobContext<'a> {
    feature_uri: &'a str,
    feature_name: &'a str,
    feature_keyword: &'a str,
    feature_location: Location,
    feature_key: usize,
    rule: Option<RuleMeta<'a>>,
    rule_key: Option<usize>,
    feature_bg: Option<&'a Background<'static>>,
    rule_bg: Option<&'a Background<'static>>,
}

/// Flatten loaded features into ordered scenario jobs.
pub(super) fn collect_jobs(features: &Features) -> Vec<ScenarioJob<'_>> {
    let mut jobs = Vec::new();
    let mut rule_key = 0usize;

    for (feature_key, doc) in features.documents().iter().enumerate() {
        let feature = &doc.feature;
        let feature_tags = tag_names(&feature.tags);
        let mut feature_bg: Option<&Background<'static>> = None;

        for child in &feature.children {
            match child {
                FeatureChild::Background(bg) => {
                    feature_bg = Some(bg);
                }
                FeatureChild::Scenario(scenario) => {
                    push_scenario_jobs(
                        &mut jobs,
                        &ScenarioJobContext {
                            feature_uri: doc.uri.as_str(),
                            feature_name: feature.name.as_ref(),
                            feature_keyword: feature.keyword.as_ref(),
                            feature_location: feature.location,
                            feature_key,
                            rule: None,
                            rule_key: None,
                            feature_bg,
                            rule_bg: None,
                        },
                        scenario,
                        &feature_tags,
                        &[],
                    );
                }
                FeatureChild::Rule(rule) => {
                    let this_rule = rule_key;
                    rule_key += 1;
                    let rule_tags = tag_names(&rule.tags);
                    let rule_meta = RuleMeta {
                        name: rule.name.as_ref(),
                        keyword: rule.keyword.as_ref(),
                        location: rule.location,
                    };
                    let mut rule_bg: Option<&Background<'static>> = None;
                    for child in &rule.children {
                        match child {
                            RuleChild::Background(bg) => rule_bg = Some(bg),
                            RuleChild::Scenario(scenario) => {
                                push_scenario_jobs(
                                    &mut jobs,
                                    &ScenarioJobContext {
                                        feature_uri: doc.uri.as_str(),
                                        feature_name: feature.name.as_ref(),
                                        feature_keyword: feature.keyword.as_ref(),
                                        feature_location: feature.location,
                                        feature_key,
                                        rule: Some(rule_meta),
                                        rule_key: Some(this_rule),
                                        feature_bg,
                                        rule_bg,
                                    },
                                    scenario,
                                    &feature_tags,
                                    &rule_tags,
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    jobs
}

fn push_scenario_jobs<'a>(
    jobs: &mut Vec<ScenarioJob<'a>>,
    ctx: &ScenarioJobContext<'a>,
    scenario: &'a Scenario<'static>,
    feature_tags: &[&str],
    rule_tags: &[&str],
) {
    let scenario_tags = tag_names(&scenario.tags);
    for (example, examples_tags) in expand_examples(scenario) {
        let effective = merge_tags(feature_tags, rule_tags, &scenario_tags, &examples_tags);
        let serial = effective.contains(&"@serial");

        let mut steps = Vec::new();
        if let Some(bg) = ctx.feature_bg {
            steps.extend(bg.steps.iter().map(|s| runnable_step(s, &example)));
        }
        if let Some(bg) = ctx.rule_bg {
            steps.extend(bg.steps.iter().map(|s| runnable_step(s, &example)));
        }
        steps.extend(scenario.steps.iter().map(|s| runnable_step(s, &example)));

        jobs.push(ScenarioJob {
            feature_uri: ctx.feature_uri,
            feature_name: ctx.feature_name,
            feature_keyword: ctx.feature_keyword,
            feature_location: ctx.feature_location,
            feature_key: ctx.feature_key,
            rule: ctx.rule,
            rule_key: ctx.rule_key,
            scenario_name: scenario.name.as_ref(),
            scenario_keyword: scenario.keyword.as_ref(),
            scenario_location: scenario.location,
            serial,
            steps,
        });
    }
}

fn runnable_step<'a>(step: &'a Step<'static>, example: &ExampleRow) -> RunnableStep<'a> {
    let substitute = example.needs_substitution();
    let text = if substitute {
        Cow::Owned(example.substitute(step.text.as_ref()).into_owned())
    } else {
        Cow::Borrowed(step.text.as_ref())
    };

    let doc_string = step.doc_string.as_ref().map(|doc| {
        let content = doc.content();
        if substitute {
            Cow::Owned(example.substitute(content.as_ref()).into_owned())
        } else {
            content
        }
    });

    let table = step.data_table.as_ref().map(|table| {
        table
            .rows
            .iter()
            .map(|row| {
                row.cells
                    .iter()
                    .map(|cell| {
                        if substitute {
                            example.substitute(cell.value.as_ref()).into_owned()
                        } else {
                            cell.value.as_ref().to_owned()
                        }
                    })
                    .collect()
            })
            .collect()
    });

    RunnableStep {
        keyword: Cow::Borrowed(step.keyword.as_ref()),
        keyword_type: step.keyword_type,
        text,
        location: step.location,
        doc_string,
        table,
    }
}

fn expand_examples<'a>(scenario: &'a Scenario<'static>) -> Vec<(ExampleRow, Vec<&'a str>)> {
    if scenario.examples.is_empty() {
        return vec![(ExampleRow::default(), Vec::new())];
    }

    let mut rows = Vec::new();
    for examples in &scenario.examples {
        let Some(header) = examples.table_header.as_ref() else {
            continue;
        };
        let examples_tags = tag_names(&examples.tags);
        let needles: Vec<String> = header
            .cells
            .iter()
            .map(|c| format!("<{}>", c.value.as_ref()))
            .collect();

        for body in &examples.table_body {
            let mut values = Vec::new();
            for (needle, cell) in needles.iter().zip(body.cells.iter()) {
                values.push((needle.clone(), cell.value.as_ref().to_owned()));
            }
            rows.push((ExampleRow { values }, examples_tags.clone()));
        }
    }

    if rows.is_empty() {
        vec![(ExampleRow::default(), Vec::new())]
    } else {
        rows
    }
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
