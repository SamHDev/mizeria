# Crate features

Mizeria’s Cargo features (from `Cargo.toml`):

| Feature | Default | Purpose |
|---|---|---|
| `dialects` | yes | Embed the full official Gherkin dialect catalog; without it, only English |
| `colour` | yes | ANSI-coloured `ColourSyncPrinter` (and the default `Printer` alias) |
| `prelude` | yes | `mizeria::prelude` re-exports |
| `regex` | yes | `Regex` / `re` step matchers (`regex` crate); without it, exact-string only |
| `json` | no | Legacy Cucumber JSON `JsonReporter` (`serde` / `serde_json`) |

## Typical dependency lines

Defaults (coloured printer + prelude + dialects + regex):

```toml
[dependencies]
mizeria = "0.1"
```

Minimal English-only, no colour, no prelude, no regex:

```toml
[dependencies]
mizeria = { version = "0.1", default-features = false }
```

JSON reporting:

```toml
[dependencies]
mizeria = { version = "0.1", features = ["json"] }
```

The `withdrawing_cash` and `cukes` examples require `prelude` and `regex`:

```toml
[[example]]
name = "withdrawing_cash"
required-features = ["prelude", "regex"]
```

## Feature interactions

- Without `colour`, `Printer` is `ASCIISyncPrinter`.
- Without `prelude`, import types from their modules (`mizeria::suite::Suite`,
  `mizeria::extract::CtxMut`, …) or enable `prelude` in examples/tests only.
- Without `regex`, register exact-string steps (`Const` / `&'static str`);
  `Regex`, `re`, and `RegexCaptures` are unavailable.
- `json` is additive; bind `JsonReporter` alone, or compose with a printer via
  `Printer::stdout().and(JsonReporter::stdout())` — a suite still holds a single
  listener slot via `bind`.
