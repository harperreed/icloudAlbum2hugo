# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build/Test Commands
- Build: `cargo build`
- Run: `cargo run`
- Test: `cargo test`
- Run specific test: `cargo test <test_name>`
- Check format: `cargo fmt --check`
- Format code: `cargo fmt`
- Lint: `cargo clippy -- -D warnings`

## Code Style Guidelines
- Follow Rust standard idioms and naming conventions
- Use `anyhow` for error handling with context
- Organize code into modules by feature (commands, icloud, photo, metadata, hugo)
- Function names: snake_case
- Type names: PascalCase
- Prefer strong typing over `String` when possible
- Document public API with rustdoc comments
- Use match expressions over if-else chains for enum handling
- Implement proper error propagation with `?` operator
- Use `serde` derive macros for serialization/deserialization
- Follow TDD practices - write tests before implementation