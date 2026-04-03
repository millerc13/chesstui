# chesstui development task runner
# Install just: https://github.com/casey/just
#   brew install just  |  cargo install just

set dotenv-load := false

binary := "chesstui"
version := `grep '^version' Cargo.toml | head -1 | cut -d'"' -f2`

# ─── Dev ────────────────────────────────────────────────────────────────

# List available recipes
default:
    @just --list

# Run all checks (what CI runs)
check: fmt-check clippy test

# Format all code
fmt:
    cargo fmt --all

# Check formatting without modifying files
fmt-check:
    cargo fmt --all --check

# Run clippy lints
clippy:
    cargo clippy --all-targets --all-features -- -D warnings

# Run tests
test:
    cargo test --all-features --workspace

# Run tests including slow/ignored tests
test-all:
    cargo test --all-features --workspace -- --ignored

# Build debug binary
build:
    cargo build

# Build release binary
build-release:
    cargo build --release

# Run the client (pass args after --)
client *ARGS:
    cargo run -- client {{ARGS}}

# Run the server (pass args after --)
server *ARGS:
    cargo run -- server {{ARGS}}

# Run the AI opponent (pass args after --)
ai *ARGS:
    cargo run -- ai {{ARGS}}

# ─── Git hooks ──────────────────────────────────────────────────────────

# Install pre-commit hooks
setup-hooks:
    bash scripts/setup-hooks.sh

# ─── Cross compilation ─────────────────────────────────────────────────

# Build for a specific target triple
build-target target:
    cargo build --release --locked --target {{target}}

# Build static Linux x86_64 binary (musl)
build-linux-x86:
    cargo build --release --locked --target x86_64-unknown-linux-musl

# Build static Linux aarch64 binary (needs cross)
build-linux-arm:
    cross build --release --locked --target aarch64-unknown-linux-musl

# Build macOS universal binary (run on macOS only)
build-macos-universal:
    cargo build --release --locked --target x86_64-apple-darwin
    cargo build --release --locked --target aarch64-apple-darwin
    mkdir -p target/universal-apple-darwin/release
    lipo -create \
        target/x86_64-apple-darwin/release/{{binary}} \
        target/aarch64-apple-darwin/release/{{binary}} \
        -output target/universal-apple-darwin/release/{{binary}}
    @echo "Universal binary: target/universal-apple-darwin/release/{{binary}}"
    lipo -info target/universal-apple-darwin/release/{{binary}}

# ─── Release ────────────────────────────────────────────────────────────

# Tag and push a release (usage: just release 0.2.0)
release ver:
    @echo "Releasing v{{ver}}..."
    @test "$(grep '^version' Cargo.toml | head -1 | cut -d'\"' -f2)" = "{{ver}}" \
        || (echo "ERROR: Cargo.toml version does not match {{ver}}" && exit 1)
    git diff --quiet || (echo "ERROR: working tree is dirty" && exit 1)
    git tag -a "v{{ver}}" -m "Release v{{ver}}"
    git push origin "v{{ver}}"
    @echo "Tag v{{ver}} pushed. GitHub Actions will handle the rest."

# ─── Deploy ─────────────────────────────────────────────────────────────

# Deploy server to production (requires DEPLOY_HOST)
deploy *ARGS:
    bash deploy/deploy.sh {{ARGS}}

# First-time server setup (run on the server itself)
setup-server:
    @echo "Run this on the target server: sudo bash deploy/setup-server.sh"

# ─── Misc ───────────────────────────────────────────────────────────────

# Remove build artifacts
clean:
    cargo clean

# Show binary size for release build
size:
    cargo build --release
    @ls -lh target/release/{{binary}} | awk '{print $5, $9}'

# Run with RUST_LOG=debug
debug-run *ARGS:
    RUST_LOG=debug cargo run -- {{ARGS}}
