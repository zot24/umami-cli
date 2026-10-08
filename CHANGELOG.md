# Changelog

All notable changes to this project are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project uses
[Semantic Versioning](https://semver.org/).

## [0.1.1] - 2026-10-08

### Added

- Two-factor login for Umami v3. When the server answers `requiresTwoFactor`,
  `auth login` asks for the 6-digit code or takes `--otp <code>` or
  `--backup-code <code>`. It then finishes the login through
  `/api/2fa/verify` and saves that token.
- `auth login --password-stdin` and the `UMAMI_PASSWORD` environment variable
  for logging in without a terminal.

### Fixed

- `auth login` no longer saves an empty token. An answer without a usable
  token is an error that names the answer's keys, never their values. The
  existing config is left untouched.
- `auth login` no longer panics without a terminal (e.g. Claude Code's `!`
  prefix). A missing server, username, password or code is an error that
  names the flag to pass.
- Server errors during login (wrong password, invalid or reused two-factor
  code, lockout with `lockedUntil`) print as readable messages.
- On Unix, `config.toml`, which holds the token, is saved with mode `0600`
  (owner only). It was created `0644`. The next login tightens an existing
  `0644` file.

### Changed

- `auth login` exits with status 1 when it fails (it exited 0).

## [0.1.0] - 2026-04-20

- Initial release.
