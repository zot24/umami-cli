use clap::Subcommand;
use serde_json::Value;

use crate::api::UmamiClient;
use crate::config::Config;
use crate::output::*;

#[derive(Subcommand)]
pub enum UsersCmd {
    /// Get current user info
    Me {
        /// Output raw JSON
        #[arg(long)]
        json: bool,
    },
    /// Get current user's teams
    MyTeams {
        /// Output raw JSON
        #[arg(long)]
        json: bool,
    },
    /// Get current user's websites
    MyWebsites {
        /// Include team websites
        #[arg(long)]
        include_teams: bool,
        /// Output raw JSON
        #[arg(long)]
        json: bool,
    },
    /// Create a user (admin only)
    Create {
        /// Username
        #[arg(long)]
        username: String,
        /// Password
        #[arg(long)]
        password: String,
        /// Role: admin, user, view-only
        #[arg(long, default_value = "user")]
        role: String,
    },
    /// Get user details (admin only)
    Get {
        /// User ID
        id: String,
        /// Output raw JSON
        #[arg(long)]
        json: bool,
    },
    /// Update a user (admin only)
    Update {
        /// User ID
        id: String,
        /// New username
        #[arg(long)]
        username: Option<String>,
        /// New password
        #[arg(long)]
        password: Option<String>,
        /// New role
        #[arg(long)]
        role: Option<String>,
    },
    /// Delete a user (admin only)
    Delete {
        /// User ID
        id: String,
    },
    /// Get user's websites (admin only)
    Websites {
        /// User ID
        id: String,
        /// Output raw JSON
        #[arg(long)]
        json: bool,
    },
    /// Get user's teams (admin only)
    Teams {
        /// User ID
        id: String,
        /// Output raw JSON
        #[arg(long)]
        json: bool,
    },
}

pub async fn run(cmd: UsersCmd) {
    let config = Config::load();
    let client = match UmamiClient::from_config(&config) {
        Ok(c) => c,
        Err(e) => {
            print_error(&e.to_string());
            return;
        }
    };

    match cmd {
        UsersCmd::Me { json } => match client.get::<Value>("/api/me").await {
            Ok(data) => {
                if json {
                    print_json(&data);
                } else {
                    println!("ID:       {}", val_str(&data, "id"));
                    println!("Username: {}", val_str(&data, "username"));
                    println!("Role:     {}", val_str(&data, "role"));
                    println!("Admin:    {}", val_str(&data, "isAdmin"));
                    println!("Created:  {}", val_str(&data, "createdAt"));
                }
            }
            Err(e) => print_error(&e.to_string()),
        },
        UsersCmd::MyTeams { json } => match client.get::<Value>("/api/me/teams").await {
            Ok(data) => {
                if json {
                    print_json(&data);
                    return;
                }
                if let Some(teams) = data.as_array() {
                    let rows: Vec<Vec<String>> = teams
                        .iter()
                        .map(|t| vec![val_str(t, "id"), val_str(t, "name"), val_str(t, "role")])
                        .collect();
                    print_table(&["ID", "NAME", "ROLE"], &rows);
                } else {
                    print_json(&data);
                }
            }
            Err(e) => print_error(&e.to_string()),
        },
        UsersCmd::MyWebsites {
            include_teams,
            json,
        } => {
            let mut query = vec![];
            if include_teams {
                query.push(("includeTeams".into(), "true".into()));
            }
            match client
                .get_with_query::<Value>("/api/me/websites", &query)
                .await
            {
                Ok(data) => {
                    if json {
                        print_json(&data);
                        return;
                    }
                    let items = data.get("data").and_then(|d| d.as_array()).or_else(|| data.as_array());
                    if let Some(sites) = items {
                        let rows: Vec<Vec<String>> = sites
                            .iter()
                            .map(|s| {
                                vec![
                                    val_str(s, "id"),
                                    val_str(s, "name"),
                                    val_str(s, "domain"),
                                ]
                            })
                            .collect();
                        print_table(&["ID", "NAME", "DOMAIN"], &rows);
                    } else {
                        print_json(&data);
                    }
                }
                Err(e) => print_error(&e.to_string()),
            }
        }
        UsersCmd::Create {
            username,
            password,
            role,
        } => {
            let body = serde_json::json!({
                "username": username,
                "password": password,
                "role": role,
            });
            match client.post::<Value>("/api/users", &body).await {
                Ok(data) => {
                    print_success(&format!("User created: {}", val_str(&data, "id")));
                    print_json(&data);
                }
                Err(e) => print_error(&e.to_string()),
            }
        }
        UsersCmd::Get { id, json } => {
            match client.get::<Value>(&format!("/api/users/{id}")).await {
                Ok(data) => {
                    if json {
                        print_json(&data);
                    } else {
                        println!("ID:       {}", val_str(&data, "id"));
                        println!("Username: {}", val_str(&data, "username"));
                        println!("Role:     {}", val_str(&data, "role"));
                        println!("Created:  {}", val_str(&data, "createdAt"));
                    }
                }
                Err(e) => print_error(&e.to_string()),
            }
        }
        UsersCmd::Update {
            id,
            username,
            password,
            role,
        } => {
            let mut body = serde_json::json!({});
            if let Some(u) = username {
                body["username"] = Value::String(u);
            }
            if let Some(p) = password {
                body["password"] = Value::String(p);
            }
            if let Some(r) = role {
                body["role"] = Value::String(r);
            }
            match client
                .post::<Value>(&format!("/api/users/{id}"), &body)
                .await
            {
                Ok(data) => {
                    print_success("User updated.");
                    print_json(&data);
                }
                Err(e) => print_error(&e.to_string()),
            }
        }
        UsersCmd::Delete { id } => {
            match client.delete::<Value>(&format!("/api/users/{id}")).await {
                Ok(_) => print_success("User deleted."),
                Err(e) => print_error(&e.to_string()),
            }
        }
        UsersCmd::Websites { id, json } => {
            match client
                .get::<Value>(&format!("/api/users/{id}/websites"))
                .await
            {
                Ok(data) => {
                    if json {
                        print_json(&data);
                    } else {
                        print_json(&data);
                    }
                }
                Err(e) => print_error(&e.to_string()),
            }
        }
        UsersCmd::Teams { id, json } => {
            match client
                .get::<Value>(&format!("/api/users/{id}/teams"))
                .await
            {
                Ok(data) => {
                    if json {
                        print_json(&data);
                    } else {
                        print_json(&data);
                    }
                }
                Err(e) => print_error(&e.to_string()),
            }
        }
    }
}
