use clap::{Args, Subcommand};
use dialoguer::{Input, Password};

use crate::api::client::session_token;
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
    let answer = client
        .login(&username, &password)
        .await
        .map_err(|e| format!("Login failed: {e}"))?;
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
