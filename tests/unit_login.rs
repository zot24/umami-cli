//! `auth login` end to end: the real binary against a wiremock Umami. HOME points
//! at a temp dir so the user's own config is never read or written. Under
//! assert_cmd the binary has no terminal, which is the no-TTY case.
#![cfg(unix)]

use std::path::PathBuf;
use std::process::Output;

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::{json, Value};
use tempfile::TempDir;
use wiremock::matchers::{body_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const PASSWORD: &str = "hunter2-password";
const TOKEN: &str = "session-jwt-value";
const PARTIAL: &str = "partial-jwt-value";
const OTP: &str = "654321";
const BACKUP: &str = "backup-code-value";
const OLD_CONFIG: &str =
    "server_url = \"https://old.example.com\"\ntoken = \"old-token\"\nusername = \"old\"\n";

/// Where `dirs::config_dir()` puts the config under a fake HOME.
fn config_path(home: &TempDir) -> PathBuf {
    let dir = if cfg!(target_os = "macos") {
        home.path().join("Library/Application Support")
    } else {
        home.path().join(".config")
    };
    dir.join("umami-cli/config.toml")
}

fn write_old_config(home: &TempDir) {
    let path = config_path(home);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, OLD_CONFIG).unwrap();
}

fn read_config(home: &TempDir) -> String {
    std::fs::read_to_string(config_path(home)).unwrap()
}

fn umami(home: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("umami-cli").unwrap();
    cmd.env("HOME", home.path())
        .env_remove("XDG_CONFIG_HOME")
        .env_remove("UMAMI_PASSWORD");
    cmd
}

fn login(home: &TempDir, server: &MockServer) -> Command {
    let mut cmd = umami(home);
    cmd.args(["auth", "login", "--server", &server.uri()])
        .args(["--username", "admin"]);
    cmd
}

fn user() -> Value {
    json!({ "id": "user-1", "username": "admin", "role": "admin", "isAdmin": true })
}

async fn mock_login(server: &MockServer, status: u16, answer: Value, times: u64) {
    Mock::given(method("POST"))
        .and(path("/api/auth/login"))
        .and(body_json(
            json!({ "username": "admin", "password": PASSWORD }),
        ))
        .respond_with(ResponseTemplate::new(status).set_body_json(answer))
        .expect(times)
        .mount(server)
        .await;
}

async fn mock_two_factor_login(server: &MockServer) {
    let answer = json!({ "requiresTwoFactor": true, "partialToken": PARTIAL });
    mock_login(server, 200, answer, 1).await;
}

/// `/api/2fa/verify` answering only the partial token and exactly `body`.
async fn mock_verify(server: &MockServer, body: Value, status: u16, answer: Value, times: u64) {
    Mock::given(method("POST"))
        .and(path("/api/2fa/verify"))
        .and(header(
            "authorization",
            format!("Bearer {PARTIAL}").as_str(),
        ))
        .and(body_json(body))
        .respond_with(ResponseTemplate::new(status).set_body_json(answer))
        .expect(times)
        .mount(server)
        .await;
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

/// Neither stream carries a password, code or token, and nothing panicked.
fn assert_clean(output: &Output) {
    let all = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        stderr(output)
    );
    for secret in [PASSWORD, TOKEN, PARTIAL, OTP, BACKUP] {
        assert!(!all.contains(secret), "output leaks {secret:?}: {all}");
    }
    assert!(!all.contains("panicked"), "panicked: {all}");
}

fn assert_saved(home: &TempDir, server: &MockServer) {
    let config = read_config(home);
    assert!(config.contains(&format!("token = \"{TOKEN}\"")), "{config}");
    assert!(
        config.contains(&format!("server_url = \"{}\"", server.uri())),
        "{config}"
    );
    assert!(config.contains("username = \"admin\""), "{config}");
}

#[test]
fn login_help_lists_non_interactive_flags() {
    let home = TempDir::new().unwrap();
    umami(&home)
        .args(["auth", "login", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--password-stdin"))
        .stdout(predicate::str::contains("--otp"))
        .stdout(predicate::str::contains("--backup-code"));
}

#[tokio::test]
async fn login_saves_token_from_plain_answer() {
    let server = MockServer::start().await;
    mock_login(&server, 200, json!({ "token": TOKEN, "user": user() }), 1).await;
    let home = TempDir::new().unwrap();

    let out = login(&home, &server)
        .env("UMAMI_PASSWORD", PASSWORD)
        .assert()
        .success()
        .stdout(predicate::str::contains("Logged in successfully."))
        .get_output()
        .clone();
    assert_clean(&out);
    assert_saved(&home, &server);

    // `auth status` reads the same isolated config back.
    umami(&home)
        .args(["auth", "status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Token:    (saved)"))
        .stdout(predicate::str::contains(home.path().to_str().unwrap()));
}

#[tokio::test]
async fn login_two_factor_with_otp() {
    let server = MockServer::start().await;
    mock_two_factor_login(&server).await;
    let answer = json!({ "token": TOKEN, "user": user() });
    mock_verify(&server, json!({ "token": OTP }), 200, answer, 1).await;
    let home = TempDir::new().unwrap();
    write_old_config(&home);

    let out = login(&home, &server)
        .args(["--password-stdin", "--otp", OTP])
        .write_stdin(format!("{PASSWORD}\n"))
        .assert()
        .success()
        .get_output()
        .clone();
    assert_clean(&out);
    assert_saved(&home, &server);
}

#[tokio::test]
async fn login_two_factor_with_backup_code() {
    let server = MockServer::start().await;
    mock_two_factor_login(&server).await;
    let answer = json!({ "token": TOKEN, "user": user() });
    mock_verify(&server, json!({ "backupCode": BACKUP }), 200, answer, 1).await;
    let home = TempDir::new().unwrap();

    let out = login(&home, &server)
        .env("UMAMI_PASSWORD", PASSWORD)
        .args(["--backup-code", BACKUP])
        .assert()
        .success()
        .get_output()
        .clone();
    assert_clean(&out);
    assert_saved(&home, &server);
}

#[tokio::test]
async fn answer_without_usable_token_leaves_config_unchanged() {
    let answers = [
        (
            json!({ "user": user(), "status": "secret-status" }),
            "status, user",
        ),
        (json!({ "token": "", "user": user() }), "token, user"),
        (json!({ "token": null }), "token"),
        (json!({ "requiresTwoFactor": true }), "requiresTwoFactor"),
        (json!([TOKEN]), "none, not a JSON object"),
    ];
    for (answer, keys) in answers {
        let server = MockServer::start().await;
        mock_login(&server, 200, answer, 1).await;
        let home = TempDir::new().unwrap();
        write_old_config(&home);

        let out = login(&home, &server)
            .env("UMAMI_PASSWORD", PASSWORD)
            .assert()
            .code(1)
            .stderr(predicate::str::contains(format!("(answer keys: {keys})")))
            .get_output()
            .clone();
        assert_clean(&out);
        assert!(!stderr(&out).contains("secret-status"), "{}", stderr(&out));
        assert_eq!(read_config(&home), OLD_CONFIG);
    }
}

#[tokio::test]
async fn no_tty_without_password_fails_cleanly() {
    let server = MockServer::start().await;
    mock_login(&server, 200, json!({ "token": TOKEN }), 0).await;
    let home = TempDir::new().unwrap();
    write_old_config(&home);

    let out = login(&home, &server)
        .assert()
        .code(1)
        .stderr(predicate::str::contains("no terminal"))
        .stderr(predicate::str::contains("--password-stdin"))
        .stderr(predicate::str::contains("UMAMI_PASSWORD"))
        .get_output()
        .clone();
    assert_clean(&out);
    assert_eq!(read_config(&home), OLD_CONFIG);
}

/// The reported failure: run through Claude Code's `!` prefix with only --server.
#[tokio::test]
async fn no_tty_without_username_fails_cleanly() {
    let server = MockServer::start().await;
    mock_login(&server, 200, json!({ "token": TOKEN }), 0).await;
    let home = TempDir::new().unwrap();

    let out = umami(&home)
        .args(["auth", "login", "--server", &server.uri()])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("--username"))
        .get_output()
        .clone();
    assert_clean(&out);
}

#[tokio::test]
async fn no_tty_two_factor_without_code_fails_cleanly() {
    let server = MockServer::start().await;
    mock_two_factor_login(&server).await;
    mock_verify(
        &server,
        json!({ "token": OTP }),
        200,
        json!({ "token": TOKEN }),
        0,
    )
    .await;
    let home = TempDir::new().unwrap();
    write_old_config(&home);

    let out = login(&home, &server)
        .env("UMAMI_PASSWORD", PASSWORD)
        .assert()
        .code(1)
        .stderr(predicate::str::contains("--otp"))
        .stderr(predicate::str::contains("--backup-code"))
        .get_output()
        .clone();
    assert_clean(&out);
    assert_eq!(read_config(&home), OLD_CONFIG);
}

#[tokio::test]
async fn password_stdin_takes_the_first_line_only() {
    let server = MockServer::start().await;
    mock_login(&server, 200, json!({ "token": TOKEN }), 1).await;
    let home = TempDir::new().unwrap();

    login(&home, &server)
        .arg("--password-stdin")
        .write_stdin(format!("{PASSWORD}\r\nsecond line\n"))
        .assert()
        .success();
    assert_saved(&home, &server);
}

#[tokio::test]
async fn malformed_otp_is_rejected_before_any_request() {
    let server = MockServer::start().await;
    mock_login(&server, 200, json!({ "token": TOKEN }), 0).await;
    let home = TempDir::new().unwrap();

    login(&home, &server)
        .env("UMAMI_PASSWORD", PASSWORD)
        .args(["--otp", "12345"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("must be 6 digits"));
}

#[tokio::test]
async fn wrong_password_is_readable() {
    let server = MockServer::start().await;
    let error = json!({ "error": {
        "message": "Unauthorized", "code": "incorrect-username-password", "status": 401 } });
    mock_login(&server, 401, error, 1).await;
    let home = TempDir::new().unwrap();
    write_old_config(&home);

    let out = login(&home, &server)
        .env("UMAMI_PASSWORD", PASSWORD)
        .assert()
        .code(1)
        .stderr(predicate::str::contains(
            "Login failed (401): Incorrect username or password.",
        ))
        .get_output()
        .clone();
    assert_clean(&out);
    assert_eq!(read_config(&home), OLD_CONFIG);
}

#[tokio::test]
async fn two_factor_server_errors_are_readable() {
    let cases = [
        (
            400,
            json!({ "error": { "message": "Invalid verification code",
                "code": "two-factor-error-invalid-code", "status": 400 } }),
            "Login failed (400): Invalid two-factor code.",
        ),
        (
            429,
            json!({ "error": { "message": "Too many failed attempts",
                "code": "two-factor-error-too-many-attempts",
                "lockedUntil": "2026-10-08T18:15:00.000Z" } }),
            "Login failed (429): Too many failed two-factor attempts. \
             Locked until 2026-10-08T18:15:00.000Z.",
        ),
        (
            401,
            json!({ "error": { "message": "Unauthorized",
                "code": "two-factor-error-invalid-partial-token", "status": 401 } }),
            "Login failed (401): The two-factor step expired or was rejected. Log in again.",
        ),
        (
            400,
            json!({ "error": { "message": "Something new", "code": "two-factor-error-new" } }),
            "Login failed (400): Something new (two-factor-error-new).",
        ),
    ];
    for (status, error, message) in cases {
        let server = MockServer::start().await;
        mock_two_factor_login(&server).await;
        mock_verify(&server, json!({ "token": OTP }), status, error, 1).await;
        let home = TempDir::new().unwrap();
        write_old_config(&home);

        let out = login(&home, &server)
            .env("UMAMI_PASSWORD", PASSWORD)
            .args(["--otp", OTP])
            .assert()
            .code(1)
            .stderr(predicate::str::contains(message))
            .get_output()
            .clone();
        assert_clean(&out);
        assert_eq!(read_config(&home), OLD_CONFIG);
    }
}
