use clap::Subcommand;
use serde_json::Value;

use crate::api::UmamiClient;
use crate::config::Config;
use crate::output::*;

#[derive(Subcommand)]
pub enum SharesCmd {
    /// Create a share page
    Create {
        /// Entity ID (website, link, or pixel ID)
        #[arg(long)]
        entity_id: String,
        /// Share type: 1=website, 2=link, 3=pixel, 4=board
        #[arg(long)]
        share_type: u32,
        /// Display name
        #[arg(long)]
        name: String,
        /// URL slug
        #[arg(long)]
        slug: String,
    },
    /// Get a share page
    Get {
        /// Share ID
        id: String,
        /// Output raw JSON
        #[arg(long)]
        json: bool,
    },
    /// Update a share page
    Update {
        /// Share ID
        id: String,
        /// New name
        #[arg(long)]
        name: Option<String>,
        /// New slug
        #[arg(long)]
        slug: Option<String>,
    },
    /// Delete a share page
    Delete {
        /// Share ID
        id: String,
    },
    /// List shares for a website
    ListForWebsite {
        /// Website ID
        website_id: String,
        /// Output raw JSON
        #[arg(long)]
        json: bool,
    },
    /// Create a share for a website
    CreateForWebsite {
        /// Website ID
        #[arg(long)]
        website_id: String,
        /// Share name
        #[arg(long)]
        name: String,
        /// Share slug
        #[arg(long)]
        slug: String,
    },
}

pub async fn run(cmd: SharesCmd) {
    let config = Config::load();
    let client = match UmamiClient::from_config(&config) {
        Ok(c) => c,
        Err(e) => {
            print_error(&e.to_string());
            return;
        }
    };

    match cmd {
        SharesCmd::Create {
            entity_id,
            share_type,
            name,
            slug,
        } => {
            let body = serde_json::json!({
                "entityId": entity_id,
                "shareType": share_type,
                "name": name,
                "slug": slug,
            });
            match client.post::<Value>("/api/share", &body).await {
                Ok(data) => {
                    print_success("Share page created.");
                    print_json(&data);
                }
                Err(e) => print_error(&e.to_string()),
            }
        }
        SharesCmd::Get { id, json } => {
            match client
                .get::<Value>(&format!("/api/share/id/{id}"))
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
        SharesCmd::Update { id, name, slug } => {
            let mut body = serde_json::json!({});
            if let Some(n) = name {
                body["name"] = Value::String(n);
            }
            if let Some(s) = slug {
                body["slug"] = Value::String(s);
            }
            match client
                .post::<Value>(&format!("/api/share/id/{id}"), &body)
                .await
            {
                Ok(data) => {
                    print_success("Share page updated.");
                    print_json(&data);
                }
                Err(e) => print_error(&e.to_string()),
            }
        }
        SharesCmd::Delete { id } => {
            match client
                .delete::<Value>(&format!("/api/share/id/{id}"))
                .await
            {
                Ok(_) => print_success("Share page deleted."),
                Err(e) => print_error(&e.to_string()),
            }
        }
        SharesCmd::ListForWebsite { website_id, json } => {
            match client
                .get::<Value>(&format!("/api/websites/{website_id}/shares"))
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
        SharesCmd::CreateForWebsite {
            website_id,
            name,
            slug,
        } => {
            let body = serde_json::json!({ "name": name, "slug": slug });
            match client
                .post::<Value>(&format!("/api/websites/{website_id}/shares"), &body)
                .await
            {
                Ok(data) => {
                    print_success("Website share created.");
                    print_json(&data);
                }
                Err(e) => print_error(&e.to_string()),
            }
        }
    }
}
