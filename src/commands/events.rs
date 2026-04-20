use clap::Subcommand;
use serde_json::Value;

use crate::api::UmamiClient;
use crate::config::Config;
use crate::output::*;

#[derive(Subcommand)]
pub enum EventsCmd {
    /// List events for a website
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
    /// Get event statistics
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
        /// Compare: prev or yoy
        #[arg(long)]
        compare: Option<String>,
        /// Output raw JSON
        #[arg(long)]
        json: bool,
    },
    /// Get event time series data
    Series {
        /// Website ID
        #[arg(long)]
        website_id: String,
        /// Start time (ms epoch)
        #[arg(long)]
        start_at: i64,
        /// End time (ms epoch)
        #[arg(long)]
        end_at: i64,
        /// Time unit
        #[arg(long, default_value = "day")]
        unit: String,
        /// Timezone
        #[arg(long, default_value = "UTC")]
        timezone: String,
        /// Output raw JSON
        #[arg(long)]
        json: bool,
    },
    /// Get event data grouped by name
    Data {
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
    /// Get event data properties
    Properties {
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
    /// Send a tracking event (no auth required)
    Send {
        /// Website ID
        #[arg(long)]
        website_id: String,
        /// URL path
        #[arg(long, default_value = "/")]
        url: String,
        /// Event name (omit for pageview)
        #[arg(long)]
        name: Option<String>,
        /// Event data as JSON string
        #[arg(long)]
        data: Option<String>,
        /// Hostname
        #[arg(long)]
        hostname: Option<String>,
        /// Referrer
        #[arg(long)]
        referrer: Option<String>,
    },
}

pub async fn run(cmd: EventsCmd) {
    let config = Config::load();
    let client = match UmamiClient::from_config(&config) {
        Ok(c) => c,
        Err(e) => {
            print_error(&e.to_string());
            return;
        }
    };

    match cmd {
        EventsCmd::List {
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
                .get_with_query::<Value>(&format!("/api/websites/{website_id}/events"), &query)
                .await
            {
                Ok(data) => {
                    if json {
                        print_json(&data);
                        return;
                    }
                    let items = data.get("data").and_then(|d| d.as_array()).or_else(|| data.as_array());
                    if let Some(events) = items {
                        let rows: Vec<Vec<String>> = events
                            .iter()
                            .map(|e| {
                                vec![
                                    val_str(e, "eventName"),
                                    val_str(e, "urlPath"),
                                    val_str(e, "createdAt"),
                                ]
                            })
                            .collect();
                        print_table(&["EVENT", "URL", "TIME"], &rows);
                    } else {
                        print_json(&data);
                    }
                }
                Err(e) => print_error(&e.to_string()),
            }
        }
        EventsCmd::Stats {
            website_id,
            start_at,
            end_at,
            compare,
            json,
        } => {
            let mut query = vec![
                ("startAt".into(), start_at.to_string()),
                ("endAt".into(), end_at.to_string()),
            ];
            if let Some(c) = compare {
                query.push(("compare".into(), c));
            }
            match client
                .get_with_query::<Value>(&format!("/api/websites/{website_id}/events/stats"), &query)
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
        EventsCmd::Series {
            website_id,
            start_at,
            end_at,
            unit,
            timezone,
            json,
        } => {
            let query = vec![
                ("startAt".into(), start_at.to_string()),
                ("endAt".into(), end_at.to_string()),
                ("unit".into(), unit),
                ("timezone".into(), timezone),
            ];
            match client
                .get_with_query::<Value>(
                    &format!("/api/websites/{website_id}/events/series"),
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
        EventsCmd::Data {
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
                    &format!("/api/websites/{website_id}/event-data"),
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
        EventsCmd::Properties {
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
                    &format!("/api/websites/{website_id}/event-data/properties"),
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
        EventsCmd::Send {
            website_id,
            url,
            name,
            data,
            hostname,
            referrer,
        } => {
            // /api/send doesn't require auth
            let mut payload = serde_json::json!({
                "payload": {
                    "website": website_id,
                    "url": url,
                }
            });
            if let Some(n) = name {
                payload["payload"]["name"] = Value::String(n);
            }
            if let Some(d) = data {
                if let Ok(parsed) = serde_json::from_str::<Value>(&d) {
                    payload["payload"]["data"] = parsed;
                }
            }
            if let Some(h) = hostname {
                payload["payload"]["hostname"] = Value::String(h);
            }
            if let Some(r) = referrer {
                payload["payload"]["referrer"] = Value::String(r);
            }

            let base_url = config
                .server_url
                .as_deref()
                .unwrap_or("http://localhost:3000");
            let http = reqwest::Client::new();
            match http
                .post(format!("{}/api/send", base_url.trim_end_matches('/')))
                .json(&payload)
                .send()
                .await
            {
                Ok(resp) if resp.status().is_success() => {
                    print_success("Event sent.");
                }
                Ok(resp) => {
                    let status = resp.status();
                    let body = resp.text().await.unwrap_or_default();
                    print_error(&format!("Failed ({status}): {body}"));
                }
                Err(e) => print_error(&e.to_string()),
            }
        }
    }
}
