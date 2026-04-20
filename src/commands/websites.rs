use clap::Subcommand;
use serde_json::Value;

use crate::api::UmamiClient;
use crate::config::Config;
use crate::output::*;

#[derive(Subcommand)]
pub enum WebsitesCmd {
    /// List all websites
    List {
        /// Search filter
        #[arg(long)]
        search: Option<String>,
        /// Include team websites
        #[arg(long)]
        include_teams: bool,
        /// Page number
        #[arg(long, default_value = "1")]
        page: u32,
        /// Output raw JSON
        #[arg(long)]
        json: bool,
    },
    /// Get website details
    Get {
        /// Website ID
        id: String,
        /// Output raw JSON
        #[arg(long)]
        json: bool,
    },
    /// Create a new website
    Create {
        /// Website name
        #[arg(long)]
        name: String,
        /// Website domain
        #[arg(long)]
        domain: String,
        /// Share ID
        #[arg(long)]
        share_id: Option<String>,
        /// Team ID
        #[arg(long)]
        team_id: Option<String>,
    },
    /// Update a website
    Update {
        /// Website ID
        id: String,
        /// New name
        #[arg(long)]
        name: Option<String>,
        /// New domain
        #[arg(long)]
        domain: Option<String>,
        /// Share ID (use "null" to unshare)
        #[arg(long)]
        share_id: Option<String>,
    },
    /// Delete a website
    Delete {
        /// Website ID
        id: String,
    },
    /// Reset all data for a website
    Reset {
        /// Website ID
        id: String,
    },
}

pub async fn run(cmd: WebsitesCmd) {
    let config = Config::load();
    let client = match UmamiClient::from_config(&config) {
        Ok(c) => c,
        Err(e) => {
            print_error(&e.to_string());
            return;
        }
    };

    match cmd {
        WebsitesCmd::List {
            search,
            include_teams,
            page,
            json,
        } => {
            let mut query = vec![("page".into(), page.to_string())];
            if let Some(s) = search {
                query.push(("search".into(), s));
            }
            if include_teams {
                query.push(("includeTeams".into(), "true".into()));
            }

            match client
                .get_with_query::<Value>("/api/websites", &query)
                .await
            {
                Ok(data) => {
                    if json {
                        print_json(&data);
                        return;
                    }
                    let items = data
                        .get("data")
                        .and_then(|d| d.as_array())
                        .or_else(|| data.as_array());
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
        WebsitesCmd::Get { id, json } => match client.get::<Value>(&format!("/api/websites/{id}")).await {
            Ok(data) => {
                if json {
                    print_json(&data);
                } else {
                    println!("ID:      {}", val_str(&data, "id"));
                    println!("Name:    {}", val_str(&data, "name"));
                    println!("Domain:  {}", val_str(&data, "domain"));
                    println!("Share:   {}", val_str(&data, "shareId"));
                    println!("Created: {}", val_str(&data, "createdAt"));
                }
            }
            Err(e) => print_error(&e.to_string()),
        },
        WebsitesCmd::Create {
            name,
            domain,
            share_id,
            team_id,
        } => {
            let mut body = serde_json::json!({ "name": name, "domain": domain });
            if let Some(s) = share_id {
                body["shareId"] = Value::String(s);
            }
            if let Some(t) = team_id {
                body["teamId"] = Value::String(t);
            }
            match client.post::<Value>("/api/websites", &body).await {
                Ok(data) => {
                    print_success(&format!("Website created: {}", val_str(&data, "id")));
                    print_json(&data);
                }
                Err(e) => print_error(&e.to_string()),
            }
        }
        WebsitesCmd::Update {
            id,
            name,
            domain,
            share_id,
        } => {
            let mut body = serde_json::json!({});
            if let Some(n) = name {
                body["name"] = Value::String(n);
            }
            if let Some(d) = domain {
                body["domain"] = Value::String(d);
            }
            if let Some(s) = share_id {
                if s == "null" {
                    body["shareId"] = Value::Null;
                } else {
                    body["shareId"] = Value::String(s);
                }
            }
            match client
                .post::<Value>(&format!("/api/websites/{id}"), &body)
                .await
            {
                Ok(data) => {
                    print_success("Website updated.");
                    print_json(&data);
                }
                Err(e) => print_error(&e.to_string()),
            }
        }
        WebsitesCmd::Delete { id } => {
            match client.delete::<Value>(&format!("/api/websites/{id}")).await {
                Ok(_) => print_success("Website deleted."),
                Err(e) => print_error(&e.to_string()),
            }
        }
        WebsitesCmd::Reset { id } => {
            match client
                .post::<Value>(&format!("/api/websites/{id}/reset"), &serde_json::json!({}))
                .await
            {
                Ok(_) => print_success("Website data reset."),
                Err(e) => print_error(&e.to_string()),
            }
        }
    }
}
