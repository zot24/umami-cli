# umami-cli

A command-line tool for managing self-hosted [Umami](https://umami.is) analytics instances. Built with Rust.

## Features

- **Authentication** - Login, logout, verify, and check auth status
- **Website management** - Create, update, delete, list, and reset websites
- **Analytics** - View stats, pageviews, metrics, active visitors, and date-range queries
- **Events** - List events, view event stats and series data
- **Sessions** - List sessions, view session stats and individual session details
- **Realtime** - Live analytics with top URLs, countries, and referrers (30-minute window)
- **Reports** - List, view, and delete reports
- **Teams** - Create, update, delete teams and manage members
- **Users** - View profile, manage users (admin only)
- **Admin** - List all users, websites, and teams (self-hosted only)
- **Shares** - Manage share pages for public access
- **Links** - Create and manage tracked links
- **Pixels** - Create and manage tracking pixels

All commands support `--json` for raw JSON output.

## Installation

### From source

```sh
cargo install --path .
```

### From git

```sh
cargo install --git https://github.com/zot24/umami-cli.git
```

## Quick start

```sh
# Login to your Umami instance
umami-cli auth login

# List your websites
umami-cli websites list

# View stats for a website
umami-cli stats summary --website-id <id> --start 2024-01-01 --end 2024-01-31

# View realtime data
umami-cli realtime --website-id <id>
```

## Authentication

`auth login` saves a session token for your server. In a terminal it prompts for anything you leave out:

```sh
umami-cli auth login --server https://analytics.example.com --username admin
```

If the account has two-factor authentication, login then asks for the 6-digit code from your authenticator app. Pass `--otp <code>` to skip the prompt, or `--backup-code <code>` to use one of your backup codes instead.

Without a terminal (scripts, CI, Claude Code's `!` prefix) nothing can be prompted, so pass everything up front:

```sh
printf '%s\n' "$UMAMI_PASSWORD" | umami-cli auth login \
  --server https://analytics.example.com --username admin --password-stdin --otp 123456
```

The password comes from, in order:

1. `--password <password>` (visible in shell history and `ps`)
2. `--password-stdin` (the first line of stdin, which must be piped)
3. the `UMAMI_PASSWORD` environment variable
4. a hidden prompt, in a terminal

Without a terminal, a missing server, username, password or two-factor code is an error that names the flag to pass. Login saves the config only once the server returns a token, so a failed login exits non-zero and leaves the old config as it was.

- `umami-cli auth status` shows the server, username and config path
- `umami-cli auth verify` checks the saved token with the server
- `umami-cli auth logout` deletes the saved credentials

## Configuration

Config is stored in your platform's config directory (`~/.config/umami-cli/config.toml` on Linux, `~/Library/Application Support/umami-cli/config.toml` on macOS; `umami-cli auth status` prints the path) and managed automatically through the `auth login` command.

```toml
server_url = "https://analytics.example.com"
token = "jwt-token"
username = "admin"
```

## Commands

```
umami-cli <command> [options]

Commands:
  auth       Authentication (login, logout, verify, status)
  websites   Manage websites (list, create, update, delete, reset, get)
  stats      View website statistics (summary, active, pageviews, metrics, date-range)
  events     Manage and track events (list, stats, series)
  sessions   View session data (list, stats, get)
  reports    Run and manage reports (list, get, delete)
  realtime   View realtime analytics
  teams      Manage teams (list, create, update, delete, join, members)
  users      User management (me, my-teams, my-websites, create, get, update, delete)
  admin      Admin operations (list-users, list-websites, list-teams)
  shares     Manage share pages (create, get, update, delete)
  links      Manage tracked links (create, get, update, delete, list)
  pixels     Manage tracking pixels (create, get, update, delete, list)
```

## Development

### Prerequisites

- Rust (edition 2021)
- Docker & Docker Compose (for E2E tests)

### Build

```sh
cargo build --release
```

### Tests

```sh
# Unit tests
cargo test

# E2E tests (requires a running Umami instance)
docker compose -f docker-compose.test.yml up -d
cargo test --test e2e -- --ignored --test-threads=1
```

## License

MIT
