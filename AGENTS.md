# Agents Instructions for goup-rs

These instructions apply to all AI-assisted contributions to `thinkgos/goup-rs`, it describes the architecture, command, and development workflow of the project.

## Project Overview

**goup** is an elegant Go version manager written in Rust. It provides cross-platform (Linux, macOS, Windows) support for managing multiple Go toolchain versions.

## Development Commands

### Build & Run

```sh
# Development build 
cargo build

# Release build (optimized)
cargo build --release

# Run directly
cargo run -- <command>

# Install locally
cargo install --path .
```

### Testing

```sh
# Run all tests
cargo test

# Run all tests all features
cargo test --all-features

# Run specific test
cargo test <test_name>

# Run tests with output
cargo test -- --nocapture

# Run tests in specific module
cargo test <module_name>::
```

### Linting & Quality

```sh
# Check without building
cargo check --all

# Format code
cargo fmt --all -- --check

# Run clippy lints
cargo clippy

# Check all targets and features
cargo clippy --all-targets --all-features -- -D warnings
```

## Architecture Overview

## Environment

- **Language**: Rust (Edition 2024, MSRV 1.94)
- **Build System**: Cargo

### Technical Stack

- **CLI Framework**: clap with derive feature
- **Package**: `goup-rs` (binary: `goup`)

### Project Structure

```text
src/
├── lib.rs              # Library entry, exports Cli and Run
├── main.rs             # Binary entry, parses args and runs
├── command/            # Command implementations
│   ├── mod.rs          # Command enum and Run impl
│   ├── cache.rs
│   ├── completion.rs
│   ├── default.rs
│   ├── env.rs
│   ├── init.rs
│   ├── install.rs
│   ├── list.rs
│   ├── oneself.rs
│   ├── remove.rs
│   ├── search.rs
│   ├── shell.rs
│   └── utils.rs
├── registries/         # Registry and index providers
│   ├── mod.rs
│   ├── registry.rs
│   ├── registry_index.rs
│   ├── go_index.rs
│   └── registry_index/
│       ├── ngx_auto_index.rs
│       ├── ngx_fancy_index.rs
│       ├── official.rs
│       └── official_git.rs
├── archived/           # Archive extraction
│   ├── mod.rs
│   ├── tgz.rs
│   └── zip.rs
├── toolchain.rs        # Toolchain management
├── version.rs          # Version parsing and matching
├── dir.rs              # Directory utilities
├── consts.rs           # Constants
└── shell.rs            # Shell integration
```

The application uses a subcommand architecture where each command is implemented as a separate module under `src/command/`. All commands implement the `Run` trait. Most commands have aliases. For example, `install` also accepts `update` and `i`. Use `goup <command> --help` to see all aliases.

The application supports multiple download backends(Official, GoDev , Golang , Mirror sites.) via the `GOUP_GO_REGISTRY_INDEX` and `GOUP_GO_REGISTRY` environment variables. Each backend is implemented as a separate module in `src/registries/registry_index/`.

## Error Handling

goup follows Rust best practices for error handling:

**Rules**:

- **anyhow::Result** for CLI binary (goup is an application, not a library)
- **ALWAYS** use `?` operator, "SUGGEST" `.context("description")` with `?` operator
- **NO unwrap()** in production code (tests only - use expect("explanation") if needed)

Examples:

```rust
use anyhow::{Context, Result};

pub fn filter_git_log(input: &str) -> Result<String> {
    let lines: Vec<_> = input
        .lines()
        .filter(|line| !line.is_empty())
        .collect();

    // ✅ RIGHT: Context on error
    let hash = extract_hash(lines[0])
        .context("Failed to extract commit hash from git log")?;

    // ✅ RIGHT: Convert on error
    let hash = extract_hash(lines[0])?;

    // ❌ WRONG: Panic in production
    let hash = extract_hash(lines[0]).unwrap();

    Ok(format!("Commit: {}", hash))
}
```

## Dependencies

Core dependencies (detail see Cargo.toml):

- clap: CLI parsing with derive macros
- clap_complete: Shell completion generation
- anyhow: Error handling
- reqwest: HTTP client (blocking, rustls-tls, json)
- serde/serde_json: Configuration and JSON parsing
- which: Locate installed programs
- dialoguer: Interactive prompts
- indicatif: Progress bars and spinners
- self_update: Self-update capability (rustls, compression)
- shadow-rs: Build-time version/git info
- env_logger: Environment-based logging (with color)
- chrono: Date/time handling
- sha2: SHA256 hashing (checksums)
- hex: Hex encoding/decoding
- flate2: Gzip compression/decompression
- tar: Tar archive extraction
- zip: ZIP archive extraction
- dirs: Standard directory paths
- semver: Semantic version parsing
- owo-colors: Terminal color support
- scraper: HTML parsing (for Go downloads index)
- dotenvy: `.env` file loading

**Build dependencies:**

- `version_check`: Rust version detection
- `shadow-rs`: Build-time code generation

**Platform-specific:**

- Windows: `junction` - Directory junction support

**Dev dependencies:**

- `tempfile`, `temp-env`: Testing utilities

## Build Optimizations

Release profile (Cargo.toml:79-84):

- `opt-level = z`: Maximum optimization
- `lto = true`: Link-time optimization
- `codegen-units = 1`: Single codegen for better optimization
- `strip = true`: Remove debug symbols
- `panic = "abort"`: Smaller binary size

## CI/CD

GitHub Actions workflow (.github/workflows/release.yml):

- Multi-platform builds (macOS, Linux x86_64/ARM64, Windows)
- DEB/RPM package generation
- Automated releases on version tags (v*)
- Checksums for binary verification

## Build Verification (Mandatory)

**CRITICAL**: After ANY Rust file edits, ALWAYS run the full quality check pipeline before committing:

```sh
cargo fmt --all -- --check && cargo clippy --all-targets --all-features -- -D warnings && cargo check --all && cargo test --all-features
```

**Rules**:

- Never commit code that hasn't passed all 4 checks
- Fix ALL clippy warnings before moving on (zero tolerance)
- If build fails, fix it immediately before continuing to next task
- Pre-commit hook will auto-enforce this (see .pre-commit)

Why: Bugs break developer productivity. Quality gates prevent regressions and maintain user trust.

## Commit Message Convention

This project follows [Conventional Commits](https://www.conventionalcommits.org/).

### Format

```
head: <type>(<scope>): <subject>
<body>
<footer>
```

**head**:

- type: feat, fix, doc, perf, style, refactor, test, chore, security, revert
- scope: can be empty (eg. if the change is a global or difficult to assign to a single component)
- subject: start with verb (such as 'change'), 50-character line

**body**: 72-character wrapped, This should answer:

- Why was this change necessary?
- How does it address the problem?
- Are there any side effects?

**footer**:

- Include a link to the ticket, if any.(`Refs: #123` or `Closes: #123`)
- BREAKING CHANGE(`BREAKING CHANGE: description`)

### Examples

```
feat(registry): add support for private mirrors

Add support for custom Go mirrors via GOUP_GO_REGISTRY environment
variable. This allows organizations to use internal mirrors for downloads.

The implementation adds a new Registry trait implementation and
updates the installer to honor the environment variable.

Refs: #42
```

## Boundaries

**Don't panic on failure** (breaks user workflow) Always use `?` operator, Log to stdout if need.
