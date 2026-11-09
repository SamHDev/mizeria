# Introduction

Cutecumber is a Cucumber-style BDD runner for Rust with **Axum-like step extractors**.
You write Gherkin `.feature` files, register `Given` / `When` / `Then` handlers on a
[`Suite`](suite.md), and run them. Handlers pull typed values from the step context —
capture groups, a scenario world, doc strings, data tables, and scenario entities —
instead of parsing step text by hand.

## Mental model

Three pieces of state show up again and again:

| Piece | Lifetime | Role |
|---|---|---|
| **World** (`CtxMut` / `CtxRef`) | One per scenario | Shared mutable bag you define (`struct World { … }`) |
| **Entity bag** | One per scenario | Typed Get / Put / Insert / Remove store; great for named domain objects |
| **Listener** | One per suite run | Observes lifecycle events (printing, metrics); does not drive steps |

World and entities are complementary. Use a world when state is a single struct you
mutate in place. Use entities when steps create, look up, and update typed values by
key (for example `"Alice"` → `Balance`). Listeners only watch.

## What this book covers

- Setting up a suite and loading features
- Matching step text with statements
- Writing handlers with extractors and entity effects
- Reporting with listeners and interpreting outcomes

API details live in rustdoc (`cargo doc --open`). This guide focuses on how the pieces
fit together.

## Building the book

From the repo root:

```bash
mdbook serve book
```

The HTML build lands in `book/html/` (gitignored).
