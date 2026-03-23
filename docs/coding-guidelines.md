# Coding Guidelines

Rust tooling enforces most conventions automatically via `rustfmt` and `clippy`.
This document covers decisions not captured by tools.

## Error Handling
- Use `thiserror` for library error types, `anyhow` for application-level errors
- Never use `unwrap()` or `expect()` in production code paths — use `?` propagation
- Return `Result<T, E>` from fallible functions
- **Exception:** `Mutex::lock().unwrap()` is acceptable — a poisoned mutex indicates
  unrecoverable state and panicking is the correct response

## Module Structure
- One struct/trait per file where practical
- Keep domain logic in `crates/domain`, infrastructure behind traits

## Testing
- Unit tests in the same file as the code under test (`#[cfg(test)]` module)
- Integration tests in `tests/` at the crate root
- Use `just test <module>` to run tests for a specific crate
