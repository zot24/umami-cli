use clap::Subcommand;
use serde_json::Value;

use crate::api::UmamiClient;
use crate::config::Config;
use crate::output::*;

#[derive(Subcommand)]
pub enum TeamsCmd {
    /// List all teams
    List {
        /// Output raw JSON
        #[arg(long)]
        json: bool,
    },
    /// Get team details
    Get {
        /// Team ID
        id: String,
        /// Output raw JSON
        #[arg(long)]
        json: bool,
    },
    /// Create a new team
    Create {
        /// Team name
        #[arg(long)]
        name: String,
    },
    /// Update a team
    Update {
        /// Team ID
        id: String,
        /// New name
        #[arg(long)]
        name: Option<String>,
    },
    /// Delete a team
    Delete {
        /// Team ID
        id: String,
    },
    /// Join a team via access code
    Join {
        /// Access code
        #[arg(long)]
        code: String,
    },
    /// List team members
    Members {
        /// Team ID
        id: String,
        /// Output raw JSON
        #[arg(long)]
        json: bool,
    },
    /// Add a member to a team
    AddMember {
        /// Team ID
        #[arg(long)]
        team_id: String,
        /// User ID
        #[arg(long)]
        user_id: String,
        /// Role: team-member, team-view-only, team-manager
        #[arg(long, default_value = "team-member")]
        role: String,
    },
    /// Update a team member's role
    UpdateMember {
        /// Team ID
        #[arg(long)]
        team_id: String,
        /// User ID
        #[arg(long)]
        user_id: String,
        /// Role: team-member, team-view-only, team-manager
        #[arg(long)]
        role: String,
    },
    /// Remove a member from a team
    RemoveMember {
        /// Team ID
        #[arg(long)]
        team_id: String,
        /// User ID
        #[arg(long)]
        user_id: String,
    },
    /// List team websites
    Websites {
        /// Team ID
        id: String,
        /// Output raw JSON
        #[arg(long)]
        json: bool,
    },
}

pub async fn run(cmd: TeamsCmd) {
    let config = Config::load();
    let client = match UmamiClient::from_config(&config) {
        Ok(c) => c,
        Err(e) => {
            print_error(&e.to_string());
            return;
        }
    };

    match cmd {
        TeamsCmd::List { json } => match client.get::<Value>("/api/teams").await {
            Ok(data) => {
                if json {
                    print_json(&data);
                    return;
                }
                let items = data.get("data").and_then(|d| d.as_array()).or_else(|| data.as_array());
                if let Some(teams) = items {
                    let rows: Vec<Vec<String>> = teams
                        .iter()
                        .map(|t| {
                            vec![
                                val_str(t, "id"),
                                val_str(t, "name"),
                                val_str(t, "accessCode"),
                                val_str(t, "createdAt"),
                            ]
                        })
                        .collect();
                    print_table(&["ID", "NAME", "ACCESS CODE", "CREATED"], &rows);
                } else {
                    print_json(&data);
                }
            }
            Err(e) => print_error(&e.to_string()),
        },
        TeamsCmd::Get { id, json } => {
            match client.get::<Value>(&format!("/api/teams/{id}")).await {
                Ok(data) => {
                    if json {
                        print_json(&data);
                    } else {
                        println!("ID:     {}", val_str(&data, "id"));
                        println!("Name:   {}", val_str(&data, "name"));
                        println!("Code:   {}", val_str(&data, "accessCode"));
                        println!("Created: {}", val_str(&data, "createdAt"));
                    }
                }
                Err(e) => print_error(&e.to_string()),
            }
        }
        TeamsCmd::Create { name } => {
            let body = serde_json::json!({ "name": name });
            match client.post::<Value>("/api/teams", &body).await {
                Ok(data) => {
                    print_success(&format!("Team created: {}", val_str(&data, "id")));
                    print_json(&data);
                }
                Err(e) => print_error(&e.to_string()),
            }
        }
        TeamsCmd::Update { id, name } => {
            let mut body = serde_json::json!({});
            if let Some(n) = name {
                body["name"] = Value::String(n);
            }
            match client
                .post::<Value>(&format!("/api/teams/{id}"), &body)
                .await
            {
                Ok(data) => {
                    print_success("Team updated.");
                    print_json(&data);
                }
                Err(e) => print_error(&e.to_string()),
            }
        }
        TeamsCmd::Delete { id } => {
            match client.delete::<Value>(&format!("/api/teams/{id}")).await {
                Ok(_) => print_success("Team deleted."),
                Err(e) => print_error(&e.to_string()),
            }
        }
        TeamsCmd::Join { code } => {
            let body = serde_json::json!({ "accessCode": code });
            match client.post::<Value>("/api/teams/join", &body).await {
                Ok(data) => {
                    print_success("Joined team.");
                    print_json(&data);
                }
                Err(e) => print_error(&e.to_string()),
            }
        }
        TeamsCmd::Members { id, json } => {
            match client
                .get::<Value>(&format!("/api/teams/{id}/users"))
                .await
            {
                Ok(data) => {
                    if json {
                        print_json(&data);
                        return;
                    }
                    if let Some(members) = data.as_array() {
                        let rows: Vec<Vec<String>> = members
                            .iter()
                            .map(|m| {
                                vec![
                                    val_str(m, "id"),
                                    val_str(m, "username"),
                                    val_str(m, "role"),
                                ]
                            })
                            .collect();
                        print_table(&["ID", "USERNAME", "ROLE"], &rows);
                    } else {
                        print_json(&data);
                    }
                }
                Err(e) => print_error(&e.to_string()),
            }
        }
        TeamsCmd::AddMember {
            team_id,
            user_id,
            role,
        } => {
            let body = serde_json::json!({ "userId": user_id, "role": role });
            match client
                .post::<Value>(&format!("/api/teams/{team_id}/users"), &body)
                .await
            {
                Ok(data) => {
                    print_success("Member added.");
                    print_json(&data);
                }
                Err(e) => print_error(&e.to_string()),
            }
        }
        TeamsCmd::UpdateMember {
            team_id,
            user_id,
            role,
        } => {
            let body = serde_json::json!({ "role": role });
            match client
                .post::<Value>(&format!("/api/teams/{team_id}/users/{user_id}"), &body)
                .await
            {
                Ok(data) => {
                    print_success("Member role updated.");
                    print_json(&data);
                }
                Err(e) => print_error(&e.to_string()),
            }
        }
        TeamsCmd::RemoveMember { team_id, user_id } => {
            match client
                .delete::<Value>(&format!("/api/teams/{team_id}/users/{user_id}"))
                .await
            {
                Ok(_) => print_success("Member removed."),
                Err(e) => print_error(&e.to_string()),
            }
        }
        TeamsCmd::Websites { id, json } => {
            match client
                .get::<Value>(&format!("/api/teams/{id}/websites"))
                .await
            {
                Ok(data) => {
                    if json {
                        print_json(&data);
                        return;
                    }
                    if let Some(sites) = data.as_array() {
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
    }
}
