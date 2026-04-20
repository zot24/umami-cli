use clap::Subcommand;
use dialoguer::{Input, Password};

use crate::api::UmamiClient;
use crate::config::Config;
use crate::output::{print_error, print_json, print_success};

#[derive(Subcommand)]
pub enum AuthCmd {
    /// Log in to your Umami instance
    Login {
        /// Server URL (e.g. https://analytics.example.com)
        #[arg(long)]
        server: Option<String>,
        /// Username
        #[arg(long)]
        username: Option<String>,
        /// Password
        #[arg(long)]
        password: Option<String>,
    },
    /// Verify current authentication token
    Verify,
    /// Log out and clear saved credentials
    Logout,
    /// Show current auth status
    Status,
}

pub async fn run(cmd: AuthCmd) {
    match cmd {
        AuthCmd::Login {
            server,
            username,
            password,
        } => {
            let server = server.unwrap_or_else(|| {
                Input::new()
                    .with_prompt("Server URL")
                    .interact_text()
                    .unwrap()
            });
            let username = username.unwrap_or_else(|| {
                Input::new()
                    .with_prompt("Username")
                    .interact_text()
                    .unwrap()
            });
            let password = password.unwrap_or_else(|| {
                Password::new()
                    .with_prompt("Password")
                    .interact()
                    .unwrap()
            });

            let mut config = Config::default();
            config.server_url = Some(server.clone());

            let mut client = match UmamiClient::from_config(&config) {
                Ok(c) => c,
                Err(e) => {
                    print_error(&e.to_string());
                    return;
                }
            };

            match client.login(&username, &password).await {
                Ok(data) => {
                    let token = data
                        .get("token")
                        .and_then(|t| t.as_str())
                        .unwrap_or_default();
                    config.token = Some(token.to_string());
                    config.username = Some(username);
                    if let Err(e) = config.save() {
                        print_error(&format!("Failed to save config: {e}"));
                        return;
                    }
                    print_success("Logged in successfully.");
                }
                Err(e) => print_error(&format!("Login failed: {e}")),
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
