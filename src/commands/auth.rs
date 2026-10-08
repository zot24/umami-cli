use clap::{Args, Subcommand};
use dialoguer::{Input, Password};

use serde_json::Value;

use crate::api::client::{session_token, two_factor_challenge, ApiError, TwoFactorCode};
use crate::api::UmamiClient;
use crate::config::Config;
use crate::output::{print_error, print_json, print_success};

#[derive(Args)]
pub struct LoginArgs {
    /// Server URL (e.g. https://analytics.example.com)
    #[arg(long)]
    server: Option<String>,
    /// Username
    #[arg(long)]
    username: Option<String>,
    /// Password
    #[arg(long)]
    password: Option<String>,
    /// Two-factor code from your authenticator app (6 digits)
    #[arg(long, value_name = "CODE", conflicts_with = "backup_code")]
    otp: Option<String>,
    /// One of your two-factor backup codes, instead of --otp
    #[arg(long, value_name = "CODE")]
    backup_code: Option<String>,
}

#[derive(Subcommand)]
pub enum AuthCmd {
    /// Log in to your Umami instance
    Login(LoginArgs),
    /// Verify current authentication token
    Verify,
    /// Log out and clear saved credentials
    Logout,
    /// Show current auth status
    Status,
}

pub async fn run(cmd: AuthCmd) {
    match cmd {
        AuthCmd::Login(args) => {
            if let Err(e) = login(args).await {
                print_error(&e);
                std::process::exit(1);
            }
        }
        AuthCmd::Verify => {
            let config = Config::load();
            let client = match UmamiClient::from_config(&config) {
                Ok(c) => c,
                Err(e) => {
                    print_error(&e.to_string());
                    return;
                }
            };
            match client.verify().await {
                Ok(data) => {
                    print_success("Token is valid.");
                    print_json(&data);
                }
                Err(e) => print_error(&format!("Verification failed: {e}")),
            }
        }
        AuthCmd::Logout => {
            if let Err(e) = Config::clear() {
                print_error(&format!("Failed to clear config: {e}"));
                return;
            }
            print_success("Logged out. Credentials cleared.");
        }
        AuthCmd::Status => {
            let config = Config::load();
            if config.token.is_some() {
                println!("Server:   {}", config.server_url.as_deref().unwrap_or("—"));
                println!("Username: {}", config.username.as_deref().unwrap_or("—"));
                println!("Token:    (saved)");
                println!("Config:   {}", Config::config_path());
            } else {
                println!("Not authenticated. Run `umami-cli auth login`.");
            }
        }
    }
}

/// Logs in and saves the session. The config is written only once the server has
/// answered with a usable token, so a failed login leaves the old one untouched.
async fn login(args: LoginArgs) -> Result<(), String> {
    // Check a code given up front before anything reaches the server.
    let code = match (args.otp, args.backup_code) {
        (Some(otp), _) => Some(TwoFactorCode::Otp(otp_code(&otp)?)),
        (None, Some(backup)) if !backup.trim().is_empty() => {
            Some(TwoFactorCode::Backup(backup.trim().to_string()))
        }
        (None, Some(_)) => return Err("The backup code is empty.".into()),
        (None, None) => None,
    };
    let server = args.server.unwrap_or_else(|| {
        Input::new()
            .with_prompt("Server URL")
            .interact_text()
            .unwrap()
    });
    let username = args.username.unwrap_or_else(|| {
        Input::new()
            .with_prompt("Username")
            .interact_text()
            .unwrap()
    });
    let password = args
        .password
        .unwrap_or_else(|| Password::new().with_prompt("Password").interact().unwrap());

    let mut client = UmamiClient::new(&server, None);
    let mut answer = client
        .login(&username, &password)
        .await
        .map_err(login_error)?;
    if let Some(partial_token) = two_factor_challenge(&answer).map_err(login_error)? {
        let code = match code {
            Some(code) => code,
            None => prompt_otp()?,
        };
        answer = client
            .verify_two_factor(&partial_token, &code)
            .await
            .map_err(login_error)?;
    }
    let token =
        session_token(&answer).map_err(|e| format!("Login failed: {e} Config left unchanged."))?;

    let config = Config {
        server_url: Some(server),
        token: Some(token),
        username: Some(username),
    };
    config
        .save()
        .map_err(|e| format!("Failed to save config: {e}"))?;
    print_success("Logged in successfully.");
    Ok(())
}

/// A TOTP code as the server takes it: exactly 6 digits.
fn otp_code(code: &str) -> Result<String, String> {
    let code = code.trim();
    if code.len() == 6 && code.bytes().all(|b| b.is_ascii_digit()) {
        Ok(code.to_string())
    } else {
        Err("The two-factor code must be 6 digits.".into())
    }
}

fn prompt_otp() -> Result<TwoFactorCode, String> {
    let otp = Password::new()
        .with_prompt("Two-factor code (6 digits)")
        .validate_with(|input: &String| otp_code(input).map(|_| ()))
        .interact()
        .map_err(|e| format!("Could not read the two-factor code: {e}"))?;
    Ok(TwoFactorCode::Otp(otp_code(&otp)?))
}

/// A readable message for a failed login or two-factor step. Umami answers
/// errors as `{ "error": { "code", "message", "lockedUntil"? } }`.
fn login_error(err: ApiError) -> String {
    let ApiError::Api { status, body } = &err else {
        return format!("Login failed: {err}");
    };
    let Some(error) = serde_json::from_str::<Value>(body)
        .ok()
        .and_then(|v| v.get("error").filter(|e| e.is_object()).cloned())
    else {
        return format!("Login failed: {err}");
    };
    let code = error
        .get("code")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let mut msg = match code {
        "incorrect-username-password" => "Incorrect username or password.".to_string(),
        "two-factor-error-invalid-code" => "Invalid two-factor code.".into(),
        "two-factor-error-invalid-backup-code" => "Invalid backup code.".into(),
        "two-factor-error-code-used" => {
            "That two-factor code was already used. Wait for the next one.".into()
        }
        "two-factor-error-too-many-attempts" => "Too many failed two-factor attempts.".into(),
        "two-factor-error-missing-token" | "two-factor-error-invalid-partial-token" => {
            "The two-factor step expired or was rejected. Log in again.".into()
        }
        "two-factor-error-not-enabled" => "Two-factor auth is not enabled for this user.".into(),
        "two-factor-error-not-configured" => {
            "Two-factor auth is not configured on the server (TWO_FACTOR_ENCRYPTION_KEY).".into()
        }
        _ => {
            let message = error.get("message").and_then(Value::as_str);
            match (message, code) {
                (Some(m), "") => m.to_string(),
                (Some(m), c) => format!("{m} ({c})."),
                (None, "") => format!("{err}"),
                (None, c) => format!("{c}."),
            }
        }
    };
    if let Some(until) = error.get("lockedUntil").and_then(Value::as_str) {
        msg.push_str(&format!(" Locked until {until}."));
    }
    format!("Login failed ({status}): {msg}")
}
