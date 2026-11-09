//! Gherkin language dialects and keyword matching.

mod data;

use data::DIALECTS;

use super::nodes::KeywordType;

/// Keyword catalog for one Gherkin language.
#[derive(Debug, Clone, Copy)]
pub struct Dialect {
    /// Language code (e.g. `"en"`, `"fr"`).
    pub code: &'static str,
    /// English name of the language.
    pub name: &'static str,
    /// Native name of the language.
    pub native: &'static str,
    /// `Feature:` keywords.
    pub feature: &'static [&'static str],
    /// `Rule:` keywords.
    pub rule: &'static [&'static str],
    /// `Background:` keywords.
    pub background: &'static [&'static str],
    /// `Scenario:` keywords.
    pub scenario: &'static [&'static str],
    /// `Scenario Outline:` keywords.
    pub scenario_outline: &'static [&'static str],
    /// `Examples:` keywords.
    pub examples: &'static [&'static str],
    /// `Given ` step keywords (include trailing space).
    pub given: &'static [&'static str],
    /// `When ` step keywords (include trailing space).
    pub when: &'static [&'static str],
    /// `Then ` step keywords (include trailing space).
    pub then: &'static [&'static str],
    /// `And ` step keywords (include trailing space).
    pub and: &'static [&'static str],
    /// `But ` step keywords (include trailing space).
    pub but: &'static [&'static str],
}

impl Dialect {
    /// Look up a dialect by language code (e.g. `"en"`, `"fr"`).
    ///
    /// Without the `dialects` feature, only `"en"` is available.
    pub fn get(code: &str) -> Option<&'static Dialect> {
        DIALECTS.iter().find(|d| d.code == code)
    }

    /// English (`"en"`) dialect. Always available, even without the `dialects` feature.
    pub fn english() -> &'static Dialect {
        Self::get("en").expect("english dialect")
    }

    /// Match `keyword:` title lines. Returns `(keyword_without_colon, trimmed_title)`.
    pub fn match_title<'a, 'k>(
        &self,
        line: &'a str,
        keywords: &'k [&'k str],
    ) -> Option<(&'k str, &'a str)> {
        let mut best: Option<(&'k str, &'a str)> = None;
        for &kw in keywords {
            let needed = kw.len() + 1; // keyword + ':'
            if line.len() >= needed
                && line.as_bytes().get(kw.len()) == Some(&b':')
                && line.starts_with(kw)
                && best.map(|(b, _)| kw.len() > b.len()).unwrap_or(true)
            {
                best = Some((kw, line[needed..].trim()));
            }
        }
        best
    }

    /// Match a step keyword (already includes trailing space in dialect data).
    pub fn match_step_keyword<'a, 'k>(
        &self,
        line: &'a str,
        keywords: &'k [&'k str],
    ) -> Option<(&'k str, &'a str)> {
        let mut best: Option<(&'k str, &'a str)> = None;
        for &kw in keywords {
            if line.starts_with(kw)
                && best.map(|(b, _)| kw.len() > b.len()).unwrap_or(true)
            {
                best = Some((kw, line[kw.len()..].trim_end()));
            }
        }
        best
    }

    /// Match a `Feature:` title line.
    pub fn match_feature<'a>(&self, line: &'a str) -> Option<(&'static str, &'a str)> {
        self.match_title(line, self.feature)
    }

    /// Match a `Rule:` title line.
    pub fn match_rule<'a>(&self, line: &'a str) -> Option<(&'static str, &'a str)> {
        self.match_title(line, self.rule)
    }

    /// Match a `Background:` title line.
    pub fn match_background<'a>(&self, line: &'a str) -> Option<(&'static str, &'a str)> {
        self.match_title(line, self.background)
    }

    /// Match a `Scenario:` or `Scenario Outline:` title line (longest keyword wins).
    pub fn match_scenario<'a>(&self, line: &'a str) -> Option<(&'static str, &'a str)> {
        let s = self.match_title(line, self.scenario);
        let o = self.match_title(line, self.scenario_outline);
        match (s, o) {
            (Some((a, ar)), Some((b, br))) => {
                if b.len() >= a.len() {
                    Some((b, br))
                } else {
                    Some((a, ar))
                }
            }
            (s, None) => s,
            (None, o) => o,
        }
    }

    /// Match an `Examples:` title line.
    pub fn match_examples<'a>(&self, line: &'a str) -> Option<(&'static str, &'a str)> {
        self.match_title(line, self.examples)
    }

    /// Match a step keyword line, returning `(type, keyword, remainder)`.
    pub fn match_step<'a>(
        &self,
        line: &'a str,
    ) -> Option<(KeywordType, &'static str, &'a str)> {
        let candidates = [
            (KeywordType::Context, self.given),
            (KeywordType::Action, self.when),
            (KeywordType::Outcome, self.then),
            (KeywordType::Conjunction, self.and),
            (KeywordType::Conjunction, self.but),
        ];
        let mut best: Option<(KeywordType, &'static str, &'a str)> = None;
        for (ty, kws) in candidates {
            if let Some((kw, rest)) = self.match_step_keyword(line, kws)
                && best.map(|(_, b, _)| kw.len() > b.len()).unwrap_or(true)
            {
                best = Some((ty, kw, rest));
            }
        }
        best
    }
}
