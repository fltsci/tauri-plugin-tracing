# Default recipe - show available commands
default:
    @just --list

# Run all checks (what CI runs)
check: fmt-check lint test

# Format all code
fmt:
    cargo fmt --all
    taplo fmt
    pnpm format

# Check formatting without modifying
fmt-check:
    cargo fmt --all -- --check
    taplo fmt --check --diff
    pnpm format:check

# Run linters
lint:
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    pnpm lint

# Run all tests
test:
    cargo nextest run --workspace --all-features
    cargo test --doc --all-features
    pnpm test

# Build everything
build:
    cargo build --all-features
    pnpm build

# Install dependencies
install:
    pnpm install

# Run the default-subscriber example
example-default:
    cd examples/default-subscriber && pnpm tauri dev

# Run the custom-subscriber example
example-custom:
    cd examples/custom-subscriber && pnpm tauri dev

# Generate docs
doc:
    cargo doc --all-features --open
