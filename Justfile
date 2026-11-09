example:
    cargo run --example withdrawing_cash

lint:
    cargo check
    cargo clippy

[script]
_ensure_mdbook:
    if ! command -v mdbook >/dev/null 2>&1; then
        cargo install mdbook
    fi
    
book: _ensure_mdbook
    mdbook serve ./book