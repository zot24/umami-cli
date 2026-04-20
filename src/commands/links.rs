use clap::Subcommand;
use serde_json::Value;

use crate::api::UmamiClient;
use crate::config::Config;
use crate::output::*;

#[derive(Subcommand)]
pub enum LinksCmd {
    /// List all links
    List {
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
    /// Get link details
    Get {
        /// Link ID
        id: String,
        /// Output raw JSON
        #[arg(long)]
        json: bool,
    },
    /// Create a link
    Create {
        /// Link name
        #[arg(long)]
        name: String,
        /// Target URL
        #[arg(long)]
        url: String,
        /// Slug (8+ characters)
        #[arg(long)]
        slug: String,
        /// Team ID
        #[arg(long)]
        team_id: Option<String>,
    },
    /// Update a link
    Update {
        /// Link ID
        id: String,
        /// New name
        #[arg(long)]
        name: Option<String>,
        /// New URL
        #[arg(long)]
        url: Option<String>,
        /// New slug (8+ characters)
        #[arg(long)]
        slug: Option<String>,
    },
    /// Delete a link
    Delete {
        /// Link ID
        id: String,
    },
}

pub async fn run(cmd: LinksCmd) {
    let config = Config::load();
    let client = match UmamiClient::from_config(&config) {
        Ok(c) => c,
        Err(e) => {
            print_error(&e.to_string());
            return;
        }
    };

    match cmd {
        LinksCmd::List { search, page, json } => {
            let mut query = vec![("page".into(), page.to_string())];
            if let Some(s) = search {
                query.push(("search".into(), s));
            }
            match client
                .get_with_query::<Value>("/api/links", &query)
                .await
            {
                Ok(data) => {
                    if json {
                        print_json(&data);
                        return;
                    }
                    let items = data.get("data").and_then(|d| d.as_array()).or_else(|| data.as_array());
                    if let Some(links) = items {
                        let rows: Vec<Vec<String>> = links
                            .iter()
                            .map(|l| {
                                vec![
                                    val_str(l, "id"),
                                    val_str(l, "name"),
                                    val_str(l, "url"),
                                    val_str(l, "slug"),
                                ]
                            })
                            .collect();
                        print_table(&["ID", "NAME", "URL", "SLUG"], &rows);
                    } else {
                        print_json(&data);
                    }
                }
                Err(e) => print_error(&e.to_string()),
            }
        }
        LinksCmd::Get { id, json } => {
            match client.get::<Value>(&format!("/api/links/{id}")).await {
                Ok(data) => {
                    if json {
                        print_json(&data);
                    } else {
                        println!("ID:   {}", val_str(&data, "id"));
                        println!("Name: {}", val_str(&data, "name"));
                        println!("URL:  {}", val_str(&data, "url"));
                        println!("Slug: {}", val_str(&data, "slug"));
                    }
                }
                Err(e) => print_error(&e.to_string()),
            }
        }
        LinksCmd::Create {
            name,
            url,
            slug,
            team_id,
        } => {
            let mut body = serde_json::json!({ "name": name, "url": url, "slug": slug });
            if let Some(t) = team_id {
                body["teamId"] = Value::String(t);
            }
            match client.post::<Value>("/api/links", &body).await {
                Ok(data) => {
                    print_success(&format!("Link created: {}", val_str(&data, "id")));
                    print_json(&data);
                }
                Err(e) => print_error(&e.to_string()),
            }
        }
        LinksCmd::Update {
            id,
            name,
            url,
            slug,
        } => {
            let mut body = serde_json::json!({});
            if let Some(n) = name {
                body["name"] = Value::String(n);
            }
            if let Some(u) = url {
                body["url"] = Value::String(u);
            }
            if let Some(s) = slug {
                body["slug"] = Value::String(s);
            }
            match client
                .post::<Value>(&format!("/api/links/{id}"), &body)
                .await
            {
                Ok(data) => {
                    print_success("Link updated.");
                    print_json(&data);
                }
                Err(e) => print_error(&e.to_string()),
            }
        }
        LinksCmd::Delete { id } => {
            match client.delete::<Value>(&format!("/api/links/{id}")).await {
                Ok(_) => print_success("Link deleted."),
                Err(e) => print_error(&e.to_string()),
            }
        }
    }
}
