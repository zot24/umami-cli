use clap::Subcommand;
use serde_json::Value;

use crate::api::UmamiClient;
use crate::config::Config;
use crate::output::*;

#[derive(Subcommand)]
pub enum AdminCmd {
    /// List all users (admin only)
    Users {
        /// Search filter
        #[arg(long)]
        search: Option<String>,
        /// Page number
        #[arg(long, default_value = "1")]
        page: u32,
        /// Output raw JSON
        #[arg(long)]
        json: bool,
    },
    /// List all websites (admin only)
    Websites {
        /// Search filter
        #[arg(long)]
        search: Option<String>,
        /// Page number
        #[arg(long, default_value = "1")]
        page: u32,
        /// Output raw JSON
        #[arg(long)]
        json: bool,
    },
    /// List all teams (admin only)
    Teams {
        /// Search filter
        #[arg(long)]
        search: Option<String>,
        /// Page number
        #[arg(long, default_value = "1")]
        page: u32,
        /// Output raw JSON
        #[arg(long)]
        json: bool,
    },
}

pub async fn run(cmd: AdminCmd) {
    let config = Config::load();
    let client = match UmamiClient::from_config(&config) {
        Ok(c) => c,
        Err(e) => {
            print_error(&e.to_string());
            return;
        }
    };

    match cmd {
        AdminCmd::Users { search, page, json } => {
            let mut query = vec![("page".into(), page.to_string())];
            if let Some(s) = search {
                query.push(("search".into(), s));
            }
            match client
                .get_with_query::<Value>("/api/admin/users", &query)
                .await
            {
                Ok(data) => {
                    if json {
                        print_json(&data);
                        return;
                    }
                    let items = data.get("data").and_then(|d| d.as_array()).or_else(|| data.as_array());
                    if let Some(users) = items {
                        let rows: Vec<Vec<String>> = users
                            .iter()
                            .map(|u| {
                                vec![
                                    val_str(u, "id"),
                                    val_str(u, "username"),
                                    val_str(u, "role"),
                                    val_str(u, "createdAt"),
                                ]
                            })
                            .collect();
                        print_table(&["ID", "USERNAME", "ROLE", "CREATED"], &rows);
                    } else {
                        print_json(&data);
                    }
                }
                Err(e) => print_error(&e.to_string()),
            }
        }
        AdminCmd::Websites { search, page, json } => {
            let mut query = vec![("page".into(), page.to_string())];
            if let Some(s) = search {
                query.push(("search".into(), s));
            }
            match client
                .get_with_query::<Value>("/api/admin/websites", &query)
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
                                    val_str(s, "createdAt"),
                                ]
                            })
                            .collect();
                        print_table(&["ID", "NAME", "DOMAIN", "CREATED"], &rows);
                    } else {
                        print_json(&data);
                    }
                }
                Err(e) => print_error(&e.to_string()),
            }
        }
        AdminCmd::Teams { search, page, json } => {
            let mut query = vec![("page".into(), page.to_string())];
            if let Some(s) = search {
                query.push(("search".into(), s));
            }
            match client
                .get_with_query::<Value>("/api/admin/teams", &query)
                .await
            {
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
            }
        }
    }
}
