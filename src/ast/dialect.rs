//! Gherkin language dialects and keyword matching.

mod data;

use data::DIALECTS;

use super::nodes::KeywordType;

#[derive(Debug, Clone, Copy)]
pub struct Dialect {
    pub code: &'static str,
    pub name: &'static str,
    pub native: &'static str,
    pub feature: &'static [&'static str],
    pub rule: &'static [&'static str],
    pub background: &'static [&'static str],
    pub scenario: &'static [&'static str],
    pub scenario_outline: &'static [&'static str],
    pub examples: &'static [&'static str],
    pub given: &'static [&'static str],
    pub when: &'static [&'static str],
    pub then: &'static [&'static str],
    pub and: &'static [&'static str],
    pub but: &'static [&'static str],
}

impl Dialect {
    /// Look up a dialect by language code (e.g. `"en"`, `"fr"`).
    ///
    /// Without the `dialects` feature, only `"en"` is available.
    pub fn get(code: &str) -> Option<&'static Dialect> {
        DIALECTS.iter().find(|d| d.code == code)
    }

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

    pub fn match_feature<'a>(&self, line: &'a str) -> Option<(&'static str, &'a str)> {
        self.match_title(line, self.feature)
    }

    pub fn match_rule<'a>(&self, line: &'a str) -> Option<(&'static str, &'a str)> {
        self.match_title(line, self.rule)
    }

    pub fn match_background<'a>(&self, line: &'a str) -> Option<(&'static str, &'a str)> {
        self.match_title(line, self.background)
    }

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

    pub fn match_examples<'a>(&self, line: &'a str) -> Option<(&'static str, &'a str)> {
        self.match_title(line, self.examples)
    }

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
            if let Some((kw, rest)) = self.match_step_keyword(line, kws) {
                if best.map(|(_, b, _)| kw.len() > b.len()).unwrap_or(true) {
                    best = Some((ty, kw, rest));
                }
            }
        }
        best
    }
}
