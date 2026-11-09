//! Step text matchers that produce [`Captures`] for extractors.

use std::collections::HashMap;

/// Positional and named capture groups from a successful statement match.
pub trait Captures<'s>: Send {
    /// Next positional capture, advancing an internal cursor.
    fn next(&mut self) -> Option<&'s str>;

    /// Named capture by group name (does not advance the cursor).
    fn named(&self, name: &str) -> Option<&'s str>;

    /// Invoke `f` for each named capture `(name, value)`.
    ///
    /// Default: no named groups. [`RegexCaptures`] overrides this so
    /// [`CaptureBag::from_captures`] can preserve names.
    fn for_each_named(&self, _f: &mut dyn FnMut(&str, &'s str)) {}
}

/// Owned capture groups so handlers can run after the match borrow ends.
#[derive(Debug, Clone, Default)]
pub struct CaptureBag {
    groups: Vec<String>,
    names: HashMap<String, String>,
}

impl CaptureBag {
    /// Empty bag with no positional or named groups.
    pub fn empty() -> Self {
        Self::default()
    }

    /// Copy remaining positional groups (and any named groups) into an owned bag.
    pub fn from_captures<'s>(mut caps: impl Captures<'s>) -> Self {
        let mut names = HashMap::new();
        caps.for_each_named(&mut |name, value| {
            names.insert(name.to_owned(), value.to_owned());
        });
        let mut groups = Vec::new();
        while let Some(value) = caps.next() {
            groups.push(value.to_owned());
        }
        Self { groups, names }
    }

    /// Preserve positional and named groups from a regex match without advancing it.
    pub fn from_regex(caps: &RegexCaptures<'_>) -> Self {
        Self {
            groups: caps.groups.iter().map(|s| (*s).to_owned()).collect(),
            names: caps
                .names
                .iter()
                .map(|(k, v)| (k.clone(), (*v).to_owned()))
                .collect(),
        }
    }

    /// Positional capture strings, in match order.
    pub fn groups(&self) -> &[String] {
        &self.groups
    }

    /// Named capture map (`(?P<name>…)` groups).
    pub fn names(&self) -> &HashMap<String, String> {
        &self.names
    }

    /// Borrowing [`Captures`] view with a fresh positional cursor.
    pub fn view(&self) -> CaptureBagView<'_> {
        CaptureBagView {
            groups: &self.groups,
            names: &self.names,
            cursor: 0,
        }
    }
}

/// [`Captures`] view over a [`CaptureBag`].
#[derive(Debug, Clone)]
pub struct CaptureBagView<'s> {
    groups: &'s [String],
    names: &'s HashMap<String, String>,
    cursor: usize,
}

impl<'s> Captures<'s> for CaptureBagView<'s> {
    fn next(&mut self) -> Option<&'s str> {
        let value = self.groups.get(self.cursor)?;
        self.cursor += 1;
        Some(value.as_str())
    }

    fn named(&self, name: &str) -> Option<&'s str> {
        self.names.get(name).map(String::as_str)
    }

    fn for_each_named(&self, f: &mut dyn FnMut(&str, &'s str)) {
        for (name, value) in self.names {
            f(name.as_str(), value.as_str());
        }
    }
}

/// Always-empty captures; used by [`Const`] matches.
#[derive(Debug, Clone, Copy, Default)]
pub struct EmptyCaptures;

impl<'s> Captures<'s> for EmptyCaptures {
    fn next(&mut self) -> Option<&'s str> {
        None
    }

    fn named(&self, _name: &str) -> Option<&'s str> {
        None
    }
}

/// Cursor over a borrowed slice of positional captures (no names).
#[derive(Debug, Clone)]
pub struct SliceCaptures<'s> {
    groups: &'s [&'s str],
    cursor: usize,
}

impl<'s> SliceCaptures<'s> {
    /// Start at the first element of `groups`.
    pub fn new(groups: &'s [&'s str]) -> Self {
        Self { groups, cursor: 0 }
    }
}

impl<'s> Captures<'s> for SliceCaptures<'s> {
    fn next(&mut self) -> Option<&'s str> {
        let value = self.groups.get(self.cursor).copied()?;
        self.cursor += 1;
        Some(value)
    }

    fn named(&self, _name: &str) -> Option<&'s str> {
        None
    }
}

/// Captures produced by a [`Regex`] statement match.
#[derive(Debug, Clone)]
pub struct RegexCaptures<'s> {
    groups: Vec<&'s str>,
    names: HashMap<String, &'s str>,
    cursor: usize,
}

impl<'s> Captures<'s> for RegexCaptures<'s> {
    fn next(&mut self) -> Option<&'s str> {
        let value = self.groups.get(self.cursor).copied()?;
        self.cursor += 1;
        Some(value)
    }

    fn named(&self, name: &str) -> Option<&'s str> {
        self.names.get(name).copied()
    }

    fn for_each_named(&self, f: &mut dyn FnMut(&str, &'s str)) {
        for (name, value) in &self.names {
            f(name.as_str(), value);
        }
    }
}

/// Matches step text and yields captures for [`crate::extract::StepContext`].
pub trait Statement: Send + Sync {
    /// Capture view borrowed from the matched step text.
    type Captures<'s>: Captures<'s>
    where
        Self: 's;

    /// Return captures when `step` fully matches this statement.
    fn matches<'s>(&self, step: &'s str) -> Option<Self::Captures<'s>>;
}

/// Exact string match; produces no capture groups.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Const {
    text: &'static str,
}

impl Const {
    /// Match steps whose text equals `text` exactly.
    pub const fn new(text: &'static str) -> Self {
        Self { text }
    }
}

impl From<&'static str> for Const {
    fn from(text: &'static str) -> Self {
        Self::new(text)
    }
}

impl Statement for &'static str {
    type Captures<'s>
        = EmptyCaptures
    where
        Self: 's;

    fn matches(&self, step: &str) -> Option<EmptyCaptures> {
        Const::new(self).matches(step)
    }
}

impl Statement for Const {
    type Captures<'s>
        = EmptyCaptures
    where
        Self: 's;

    fn matches(&self, step: &str) -> Option<EmptyCaptures> {
        (self.text == step).then_some(EmptyCaptures)
    }
}

/// Regular-expression match; capture groups feed extractors.
#[derive(Debug, Clone)]
pub struct Regex {
    inner: regex::Regex,
}

impl Regex {
    /// Compile `pattern` as a step regex.
    pub fn new(pattern: &str) -> Result<Self, regex::Error> {
        Ok(Self {
            inner: regex::Regex::new(pattern)?,
        })
    }
}

/// Compile a step regex, panicking with a clear message if the pattern is invalid.
///
/// Prefer this over `Regex::new(...).unwrap()` when registering steps at startup.
#[track_caller]
pub fn re(pattern: &str) -> Regex {
    match Regex::new(pattern) {
        Ok(regex) => regex,
        Err(err) => panic!("invalid step regex `{pattern}`: {err}"),
    }
}

impl Statement for Regex {
    type Captures<'s>
        = RegexCaptures<'s>
    where
        Self: 's;

    fn matches<'s>(&self, step: &'s str) -> Option<RegexCaptures<'s>> {
        let caps = self.inner.captures(step)?;
        let full = caps.get(0)?;
        if full.start() != 0 || full.end() != step.len() {
            return None;
        }

        let mut groups = Vec::new();
        let mut names = HashMap::new();

        for (i, name) in self.inner.capture_names().enumerate().skip(1) {
            let Some(m) = caps.get(i) else {
                continue;
            };
            let value = m.as_str();
            groups.push(value);
            if let Some(name) = name {
                names.insert(name.to_owned(), value);
            }
        }

        Some(RegexCaptures {
            groups,
            names,
            cursor: 0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn const_matches_exact_text() {
        let stmt = Const::new("I have 42 cukes");
        let mut caps = stmt.matches("I have 42 cukes").unwrap();
        assert!(caps.next().is_none());
        assert!(stmt.matches("I have 41 cukes").is_none());
    }

    #[test]
    fn regex_full_match_yields_positional_groups() {
        let stmt = Regex::new(r"I have (\d+) cukes in my (\w+)").unwrap();
        let mut caps = stmt.matches("I have 42 cukes in my belly").unwrap();
        assert_eq!(caps.next(), Some("42"));
        assert_eq!(caps.next(), Some("belly"));
        assert!(caps.next().is_none());
    }

    #[test]
    fn regex_named_groups() {
        let stmt = Regex::new(r"I have (?P<count>\d+) cukes").unwrap();
        let caps = stmt.matches("I have 7 cukes").unwrap();
        assert_eq!(caps.named("count"), Some("7"));
        assert!(caps.named("missing").is_none());
    }

    #[test]
    fn capture_bag_preserves_named_groups() {
        let stmt = Regex::new(r"I have (?P<count>\d+) cukes").unwrap();
        let caps = stmt.matches("I have 7 cukes").unwrap();
        let bag = CaptureBag::from_captures(caps);
        assert_eq!(bag.names().get("count").map(String::as_str), Some("7"));
        assert_eq!(bag.view().named("count"), Some("7"));
    }

    #[test]
    fn regex_rejects_partial_match() {
        let stmt = Regex::new(r"foo").unwrap();
        assert!(stmt.matches("foobar").is_none());
        assert!(stmt.matches("foo").is_some());
    }

    #[test]
    fn regex_invalid_pattern() {
        assert!(Regex::new(r"(unterminated").is_err());
    }

    #[test]
    fn re_compiles_valid_pattern() {
        let stmt = re(r"I have (\d+) cukes");
        assert!(stmt.matches("I have 3 cukes").is_some());
    }

    #[test]
    #[should_panic(expected = "invalid step regex `(unterminated`:")]
    fn re_panics_on_invalid_pattern() {
        let _ = re(r"(unterminated");
    }

    #[test]
    fn slice_captures_next() {
        let groups = ["a", "b"];
        let mut caps = SliceCaptures::new(&groups);
        assert_eq!(caps.next(), Some("a"));
        assert_eq!(caps.next(), Some("b"));
        assert!(caps.next().is_none());
        assert!(caps.named("x").is_none());
    }
}
