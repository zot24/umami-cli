use clap::Subcommand;
use serde_json::Value;

use crate::api::UmamiClient;
use crate::config::Config;
use crate::output::*;

#[derive(Subcommand)]
pub enum PixelsCmd {
    /// List all pixels
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
    /// Get pixel details
    Get {
        /// Pixel ID
        id: String,
        /// Output raw JSON
        #[arg(long)]
        json: bool,
    },
    /// Create a pixel
    Create {
        /// Pixel name
        #[arg(long)]
        name: String,
        /// Slug (8+ characters)
        #[arg(long)]
        slug: String,
        /// Team ID
        #[arg(long)]
        team_id: Option<String>,
    },
    /// Update a pixel
    Update {
        /// Pixel ID
        id: String,
        /// New name
        #[arg(long)]
        name: Option<String>,
        /// New slug (8+ characters)
        #[arg(long)]
        slug: Option<String>,
    },
    /// Delete a pixel
    Delete {
        /// Pixel ID
        id: String,
    },
}

pub async fn run(cmd: PixelsCmd) {
    let config = Config::load();
    let client = match UmamiClient::from_config(&config) {
        Ok(c) => c,
        Err(e) => {
            print_error(&e.to_string());
            return;
        }
    };

    match cmd {
        PixelsCmd::List { search, page, json } => {
            let mut query = vec![("page".into(), page.to_string())];
            if let Some(s) = search {
                query.push(("search".into(), s));
            }
            match client
                .get_with_query::<Value>("/api/pixels", &query)
                .await
            {
                Ok(data) => {
                    if json {
                        print_json(&data);
                        return;
                    }
                    let items = data.get("data").and_then(|d| d.as_array()).or_else(|| data.as_array());
                    if let Some(pixels) = items {
                        let rows: Vec<Vec<String>> = pixels
                            .iter()
                            .map(|p| {
                                vec![
                                    val_str(p, "id"),
                                    val_str(p, "name"),
                                    val_str(p, "slug"),
                                ]
                            })
                            .collect();
                        print_table(&["ID", "NAME", "SLUG"], &rows);
                    } else {
                        print_json(&data);
                    }
                }
                Err(e) => print_error(&e.to_string()),
            }
        }
        PixelsCmd::Get { id, json } => {
            match client.get::<Value>(&format!("/api/pixels/{id}")).await {
                Ok(data) => {
                    if json {
                        print_json(&data);
                    } else {
                        println!("ID:   {}", val_str(&data, "id"));
                        println!("Name: {}", val_str(&data, "name"));
                        println!("Slug: {}", val_str(&data, "slug"));
                    }
                }
                Err(e) => print_error(&e.to_string()),
            }
        }
        PixelsCmd::Create {
            name,
            slug,
            team_id,
        } => {
            let mut body = serde_json::json!({ "name": name, "slug": slug });
            if let Some(t) = team_id {
                body["teamId"] = Value::String(t);
            }
            match client.post::<Value>("/api/pixels", &body).await {
                Ok(data) => {
                    print_success(&format!("Pixel created: {}", val_str(&data, "id")));
                    print_json(&data);
                }
                Err(e) => print_error(&e.to_string()),
            }
        }
        PixelsCmd::Update { id, name, slug } => {
            let mut body = serde_json::json!({});
            if let Some(n) = name {
                body["name"] = Value::String(n);
            }
            if let Some(s) = slug {
                body["slug"] = Value::String(s);
            }
            match client
                .post::<Value>(&format!("/api/pixels/{id}"), &body)
                .await
            {
                Ok(data) => {
                    print_success("Pixel updated.");
                    print_json(&data);
                }
                Err(e) => print_error(&e.to_string()),
            }
        }
        PixelsCmd::Delete { id } => {
            match client
                .delete::<Value>(&format!("/api/pixels/{id}"))
                .await
            {
                Ok(_) => print_success("Pixel deleted."),
                Err(e) => print_error(&e.to_string()),
            }
        }
    }
}
