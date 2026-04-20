use assert_cmd::Command;
use predicates::prelude::*;

fn cli() -> Command {
    Command::cargo_bin("umami-cli").unwrap()
}

#[test]
fn shows_help() {
    cli()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("CLI tool for managing self-hosted Umami"));
}

#[test]
fn shows_version() {
    cli()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("umami-cli"));
}

#[test]
fn auth_subcommand_help() {
    cli()
        .args(["auth", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("login"))
        .stdout(predicate::str::contains("logout"))
        .stdout(predicate::str::contains("verify"))
        .stdout(predicate::str::contains("status"));
}

#[test]
fn websites_subcommand_help() {
    cli()
        .args(["websites", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("delete"))
        .stdout(predicate::str::contains("reset"));
}

#[test]
fn stats_subcommand_help() {
    cli()
        .args(["stats", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("summary"))
        .stdout(predicate::str::contains("active"))
        .stdout(predicate::str::contains("pageviews"))
        .stdout(predicate::str::contains("metrics"));
}

#[test]
fn events_subcommand_help() {
    cli()
        .args(["events", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("send"))
        .stdout(predicate::str::contains("stats"));
}

#[test]
fn sessions_subcommand_help() {
    cli()
        .args(["sessions", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("activity"))
        .stdout(predicate::str::contains("weekly"));
}

#[test]
fn reports_subcommand_help() {
    cli()
        .args(["reports", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("funnel"))
        .stdout(predicate::str::contains("retention"))
        .stdout(predicate::str::contains("journey"))
        .stdout(predicate::str::contains("attribution"))
        .stdout(predicate::str::contains("revenue"))
        .stdout(predicate::str::contains("utm"))
        .stdout(predicate::str::contains("performance"));
}

#[test]
fn realtime_subcommand_help() {
    cli()
        .args(["realtime", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("get"));
}

#[test]
fn teams_subcommand_help() {
    cli()
        .args(["teams", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("join"))
        .stdout(predicate::str::contains("members"));
}

#[test]
fn users_subcommand_help() {
    cli()
        .args(["users", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("me"))
        .stdout(predicate::str::contains("create"));
}

#[test]
fn admin_subcommand_help() {
    cli()
        .args(["admin", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("users"))
        .stdout(predicate::str::contains("websites"))
        .stdout(predicate::str::contains("teams"));
}

#[test]
fn shares_subcommand_help() {
    cli()
        .args(["shares", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("delete"));
}

#[test]
fn links_subcommand_help() {
    cli()
        .args(["links", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("create"));
}

#[test]
fn pixels_subcommand_help() {
    cli()
        .args(["pixels", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("create"));
}

#[test]
fn unknown_subcommand_fails() {
    cli()
        .arg("nonexistent")
        .assert()
        .failure();
}

#[test]
fn auth_status_when_not_logged_in() {
    cli()
        .args(["auth", "status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Not authenticated").or(predicate::str::contains("Server:")));
}
