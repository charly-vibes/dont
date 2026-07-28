## 1. Bump dependency
- [x] Bump `genesis` from `v0.1.0` to `v0.2.0` in `Cargo.toml`.

## 2. Adopt genesis::config
- [x] If the tool has `src/config.rs`, thin it to just the struct + `ConfigFile` impl.
      Otherwise, add a minimal config struct implementing `ConfigFile`.
- [x] Register the config struct with `ConfigRegistry` at startup.
- [x] Remove dead config parsing code (if any).
- [x] `cargo test` passes with the new config setup.

## 3. Adopt genesis::guide
- [x] Replace `main.rs` CLI setup with `Guide::builder(...)`.
- [x] Convert command handlers to return `Output<T>` and use `ErrorSink` for errors.
      *(Adopted the ErrorSink branch of the spec: dont keeps its envelope-aware
      error printing, but routes unknown-argument errors through `guide.error_sink()`
      and writes the feedback scratch on every non-zero exit via the ErrorSink
      contract, so `dont feedback bug --from-last-error` works. Full `Output<T>`
      conversion of all command handlers is deferred — the spec allows the
      ErrorSink alternative.)*
- [x] Remove dead error-handling code.
      *(No bespoke error code became dead: the typo path still needs
      `dont_command_registry()`/`SuggestionEngine` for envelope-mode errors that
      have no access to the Guide. Clippy is clean.)*
- [x] `cargo test` passes with the new guide setup.

## 4. Clean up
- [ ] `cargo test` passes.
- [ ] `cargo clippy` introduces no new warnings.
- [ ] `cargo fmt` is clean.
