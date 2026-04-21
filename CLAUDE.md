# umami-cli

Rust CLI for managing self-hosted Umami analytics instances. Wraps the Umami REST API.

## Build & run

```sh
cargo build                # debug build
cargo build --release      # release build
cargo run -- <command>     # run directly
```

## Test

```sh
cargo test                 # unit tests (wiremock-based HTTP mocks + assert_cmd CLI tests)

# E2E tests require a running Umami instance via Docker
docker compose -f docker-compose.test.yml up -d
cargo test --test e2e -- --ignored --test-threads=1
# Test credentials: admin / umami at http://localhost:3099
```

## Project structure

- `src/main.rs` - CLI entry point, clap command definitions
- `src/commands/` - One module per command group (auth, websites, stats, events, sessions, reports, realtime, teams, users, admin, shares, links, pixels)
- `src/api/client.rs` - HTTP client (`UmamiClient`) with auth, GET/POST/DELETE helpers
- `src/config.rs` - Config management (`~/.config/umami-cli/config.toml`)
- `src/output.rs` - Output formatting (JSON, tables, success/error messages)
- `tests/` - Unit tests (`unit_*.rs`), E2E tests (`e2e.rs`), helpers

## Key conventions

- All commands support `--json` flag for raw JSON output
- Authentication uses JWT bearer tokens stored in config.toml
- HTTP via reqwest with rustls (no OpenSSL dependency)
- Async runtime: tokio
- Error types use `thiserror` derive macros
- E2E tests are `#[ignore]` and require `--test-threads=1` (serial execution)
