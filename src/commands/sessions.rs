use clap::Subcommand;
use serde_json::Value;

use crate::api::UmamiClient;
use crate::config::Config;
use crate::output::*;

#[derive(Subcommand)]
pub enum SessionsCmd {
    /// List sessions for a website
    List {
        /// Website ID
        #[arg(long)]
        website_id: String,
        /// Start time (ms epoch)
        #[arg(long)]
        start_at: i64,
        /// End time (ms epoch)
        #[arg(long)]
        end_at: i64,
        /// Search text
        #[arg(long)]
        search: Option<String>,
        /// Page number
        #[arg(long, default_value = "1")]
        page: u32,
        /// Output raw JSON
        #[arg(long)]
        json: bool,
    },
    /// Get session statistics
    Stats {
        /// Website ID
        #[arg(long)]
        website_id: String,
        /// Start time (ms epoch)
        #[arg(long)]
        start_at: i64,
        /// End time (ms epoch)
        #[arg(long)]
        end_at: i64,
        /// Output raw JSON
        #[arg(long)]
        json: bool,
    },
    /// Get individual session details
    Get {
        /// Website ID
        #[arg(long)]
        website_id: String,
        /// Session ID
        session_id: String,
        /// Output raw JSON
        #[arg(long)]
        json: bool,
    },
    /// Get session activity
    Activity {
        /// Website ID
        #[arg(long)]
        website_id: String,
        /// Session ID
        session_id: String,
        /// Output raw JSON
        #[arg(long)]
        json: bool,
    },
    /// Get session properties
    Properties {
        /// Website ID
        #[arg(long)]
        website_id: String,
        /// Session ID
        session_id: String,
        /// Output raw JSON
        #[arg(long)]
        json: bool,
    },
    /// Get sessions by hour of weekday
    Weekly {
        /// Website ID
        #[arg(long)]
        website_id: String,
        /// Start time (ms epoch)
        #[arg(long)]
        start_at: i64,
        /// End time (ms epoch)
        #[arg(long)]
        end_at: i64,
        /// Timezone
        #[arg(long, default_value = "UTC")]
        timezone: String,
        /// Output raw JSON
        #[arg(long)]
        json: bool,
    },
}

pub async fn run(cmd: SessionsCmd) {
    let config = Config::load();
    let client = match UmamiClient::from_config(&config) {
        Ok(c) => c,
        Err(e) => {
            print_error(&e.to_string());
            return;
        }
    };

    match cmd {
        SessionsCmd::List {
            website_id,
            start_at,
            end_at,
            search,
            page,
            json,
        } => {
            let mut query = vec![
                ("startAt".into(), start_at.to_string()),
                ("endAt".into(), end_at.to_string()),
                ("page".into(), page.to_string()),
            ];
            if let Some(s) = search {
                query.push(("search".into(), s));
            }
            match client
                .get_with_query::<Value>(&format!("/api/websites/{website_id}/sessions"), &query)
                .await
            {
                Ok(data) => {
                    if json {
                        print_json(&data);
                        return;
                    }
                    let items = data.get("data").and_then(|d| d.as_array());
                    if let Some(sessions) = items {
                        let rows: Vec<Vec<String>> = sessions
                            .iter()
                            .map(|s| {
                                vec![
                                    val_str(s, "id"),
                                    val_str(s, "browser"),
                                    val_str(s, "os"),
                                    val_str(s, "country"),
                                    val_str(s, "visits"),
                                    val_str(s, "views"),
                                ]
                            })
                            .collect();
                        print_table(
                            &["ID", "BROWSER", "OS", "COUNTRY", "VISITS", "VIEWS"],
                            &rows,
                        );
                    } else {
                        print_json(&data);
                    }
                }
                Err(e) => print_error(&e.to_string()),
            }
        }
        SessionsCmd::Stats {
            website_id,
            start_at,
            end_at,
            json,
        } => {
            let query = vec![
                ("startAt".into(), start_at.to_string()),
                ("endAt".into(), end_at.to_string()),
            ];
            match client
                .get_with_query::<Value>(
                    &format!("/api/websites/{website_id}/sessions/stats"),
                    &query,
                )
                .await
            {
                Ok(data) => {
                    if json {
                        print_json(&data);
                    } else {
                        println!("Pageviews: {}", val_str(&data, "pageviews"));
                        println!("Visitors:  {}", val_str(&data, "visitors"));
                        println!("Visits:    {}", val_str(&data, "visits"));
                        println!("Countries: {}", val_str(&data, "countries"));
                        println!("Events:    {}", val_str(&data, "events"));
                    }
                }
                Err(e) => print_error(&e.to_string()),
            }
        }
        SessionsCmd::Get {
            website_id,
            session_id,
            json,
        } => {
            match client
                .get::<Value>(&format!(
                    "/api/websites/{website_id}/sessions/{session_id}"
                ))
                .await
            {
                Ok(data) => {
                    if json {
                        print_json(&data);
                    } else {
                        println!("ID:       {}", val_str(&data, "id"));
                        println!("Browser:  {}", val_str(&data, "browser"));
                        println!("OS:       {}", val_str(&data, "os"));
                        println!("Device:   {}", val_str(&data, "device"));
                        println!("Country:  {}", val_str(&data, "country"));
                        println!("Language: {}", val_str(&data, "language"));
                        println!("Visits:   {}", val_str(&data, "visits"));
                        println!("Views:    {}", val_str(&data, "views"));
                    }
                }
                Err(e) => print_error(&e.to_string()),
            }
        }
        SessionsCmd::Activity {
            website_id,
            session_id,
            json,
        } => {
            match client
                .get::<Value>(&format!(
                    "/api/websites/{website_id}/sessions/{session_id}/activity"
                ))
                .await
            {
                Ok(data) => {
                    if json {
                        print_json(&data);
                        return;
                    }
                    if let Some(items) = data.as_array() {
                        let rows: Vec<Vec<String>> = items
                            .iter()
                            .map(|a| {
                                vec![
                                    val_str(a, "createdAt"),
                                    val_str(a, "eventType"),
                                    val_str(a, "eventName"),
                                    val_str(a, "urlPath"),
                                ]
                            })
                            .collect();
                        print_table(&["TIME", "TYPE", "EVENT", "URL"], &rows);
                    } else {
                        print_json(&data);
                    }
                }
                Err(e) => print_error(&e.to_string()),
            }
        }
        SessionsCmd::Properties {
            website_id,
            session_id,
            json,
        } => {
            match client
                .get::<Value>(&format!(
                    "/api/websites/{website_id}/sessions/{session_id}/properties"
                ))
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
        SessionsCmd::Weekly {
            website_id,
            start_at,
            end_at,
            timezone,
            json,
        } => {
            let query = vec![
                ("startAt".into(), start_at.to_string()),
                ("endAt".into(), end_at.to_string()),
                ("timezone".into(), timezone),
            ];
            match client
                .get_with_query::<Value>(
                    &format!("/api/websites/{website_id}/sessions/weekly"),
                    &query,
                )
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
