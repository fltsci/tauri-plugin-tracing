---
"tracing": patch
"tracing-js": patch
---

Pin Rust toolchain to 1.95.0 via `rust-toolchain.toml`.

Different rustfmt builds can produce different layouts even with the same `--edition` flag, which surfaces as "works locally, fails CI" formatter drift. Pinning the channel makes both contributor environments and CI deterministic.

Profile is `default`, which includes `rustfmt` and `clippy` (plus rustc, cargo, rust-std, rust-docs). Contributors with `rustup` installed will auto-fetch 1.95.0 on first cargo invocation in the project directory; no extra setup required.
