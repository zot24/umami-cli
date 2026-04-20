use clap::Subcommand;
use serde_json::Value;

use crate::api::UmamiClient;
use crate::config::Config;
use crate::output::*;

#[derive(Subcommand)]
pub enum ReportsCmd {
    /// List all reports
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
    /// Get a specific report
    Get {
        /// Report ID
        id: String,
        /// Output raw JSON
        #[arg(long)]
        json: bool,
    },
    /// Delete a report
    Delete {
        /// Report ID
        id: String,
    },
    /// Run an attribution report
    Attribution {
        /// Website ID
        #[arg(long)]
        website_id: String,
        /// Start date (ISO 8601)
        #[arg(long)]
        start_date: String,
        /// End date (ISO 8601)
        #[arg(long)]
        end_date: String,
        /// Timezone
        #[arg(long, default_value = "UTC")]
        timezone: String,
        /// Output raw JSON
        #[arg(long)]
        json: bool,
    },
    /// Run a funnel report
    Funnel {
        /// Website ID
        #[arg(long)]
        website_id: String,
        /// Start date (ISO 8601)
        #[arg(long)]
        start_date: String,
        /// End date (ISO 8601)
        #[arg(long)]
        end_date: String,
        /// Funnel URLs (comma-separated)
        #[arg(long)]
        urls: String,
        /// Window in days
        #[arg(long, default_value = "7")]
        window: u32,
        /// Timezone
        #[arg(long, default_value = "UTC")]
        timezone: String,
        /// Output raw JSON
        #[arg(long)]
        json: bool,
    },
    /// Run a retention report
    Retention {
        /// Website ID
        #[arg(long)]
        website_id: String,
        /// Start date (ISO 8601)
        #[arg(long)]
        start_date: String,
        /// End date (ISO 8601)
        #[arg(long)]
        end_date: String,
        /// Timezone
        #[arg(long, default_value = "UTC")]
        timezone: String,
        /// Output raw JSON
        #[arg(long)]
        json: bool,
    },
    /// Run a journey report
    Journey {
        /// Website ID
        #[arg(long)]
        website_id: String,
        /// Start date (ISO 8601)
        #[arg(long)]
        start_date: String,
        /// End date (ISO 8601)
        #[arg(long)]
        end_date: String,
        /// Number of steps (3-7)
        #[arg(long, default_value = "5")]
        steps: u32,
        /// Timezone
        #[arg(long, default_value = "UTC")]
        timezone: String,
        /// Output raw JSON
        #[arg(long)]
        json: bool,
    },
    /// Run a revenue report
    Revenue {
        /// Website ID
        #[arg(long)]
        website_id: String,
        /// Start date (ISO 8601)
        #[arg(long)]
        start_date: String,
        /// End date (ISO 8601)
        #[arg(long)]
        end_date: String,
        /// Timezone
        #[arg(long, default_value = "UTC")]
        timezone: String,
        /// Output raw JSON
        #[arg(long)]
        json: bool,
    },
    /// Run a UTM report
    Utm {
        /// Website ID
        #[arg(long)]
        website_id: String,
        /// Start date (ISO 8601)
        #[arg(long)]
        start_date: String,
        /// End date (ISO 8601)
        #[arg(long)]
        end_date: String,
        /// Timezone
        #[arg(long, default_value = "UTC")]
        timezone: String,
        /// Output raw JSON
        #[arg(long)]
        json: bool,
    },
    /// Run a breakdown report
    Breakdown {
        /// Website ID
        #[arg(long)]
        website_id: String,
        /// Start date (ISO 8601)
        #[arg(long)]
        start_date: String,
        /// End date (ISO 8601)
        #[arg(long)]
        end_date: String,
        /// Timezone
        #[arg(long, default_value = "UTC")]
        timezone: String,
        /// Output raw JSON
        #[arg(long)]
        json: bool,
    },
    /// Run a goal report
    Goal {
        /// Website ID
        #[arg(long)]
        website_id: String,
        /// Start date (ISO 8601)
        #[arg(long)]
        start_date: String,
        /// End date (ISO 8601)
        #[arg(long)]
        end_date: String,
        /// Timezone
        #[arg(long, default_value = "UTC")]
        timezone: String,
        /// Output raw JSON
        #[arg(long)]
        json: bool,
    },
    /// Run a performance (Core Web Vitals) report
    Performance {
        /// Website ID
        #[arg(long)]
        website_id: String,
        /// Start date (ISO 8601)
        #[arg(long)]
        start_date: String,
        /// End date (ISO 8601)
        #[arg(long)]
        end_date: String,
        /// Timezone
        #[arg(long, default_value = "UTC")]
        timezone: String,
        /// Output raw JSON
        #[arg(long)]
        json: bool,
    },
}

pub async fn run(cmd: ReportsCmd) {
    let config = Config::load();
    let client = match UmamiClient::from_config(&config) {
        Ok(c) => c,
        Err(e) => {
            print_error(&e.to_string());
            return;
        }
    };

    match cmd {
        ReportsCmd::List { search, page, json } => {
            let mut query = vec![("page".into(), page.to_string())];
            if let Some(s) = search {
                query.push(("search".into(), s));
            }
            match client
                .get_with_query::<Value>("/api/reports", &query)
                .await
            {
                Ok(data) => {
                    if json {
                        print_json(&data);
                        return;
                    }
                    let items = data.get("data").and_then(|d| d.as_array()).or_else(|| data.as_array());
                    if let Some(reports) = items {
                        let rows: Vec<Vec<String>> = reports
                            .iter()
                            .map(|r| {
                                vec![
                                    val_str(r, "id"),
                                    val_str(r, "type"),
                                    val_str(r, "name"),
                                    val_str(r, "createdAt"),
                                ]
                            })
                            .collect();
                        print_table(&["ID", "TYPE", "NAME", "CREATED"], &rows);
                    } else {
                        print_json(&data);
                    }
                }
                Err(e) => print_error(&e.to_string()),
            }
        }
        ReportsCmd::Get { id, json } => {
            match client.get::<Value>(&format!("/api/reports/{id}")).await {
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
        ReportsCmd::Delete { id } => {
            match client.delete::<Value>(&format!("/api/reports/{id}")).await {
                Ok(_) => print_success("Report deleted."),
                Err(e) => print_error(&e.to_string()),
            }
        }
        ReportsCmd::Attribution {
            website_id,
            start_date,
            end_date,
            timezone,
            json,
        } => {
            run_report(&client, "attribution", &website_id, &start_date, &end_date, &timezone, None, json).await;
        }
        ReportsCmd::Funnel {
            website_id,
            start_date,
            end_date,
            urls,
            window,
            timezone,
            json,
        } => {
            let urls_vec: Vec<&str> = urls.split(',').map(|s| s.trim()).collect();
            let extra = serde_json::json!({ "urls": urls_vec, "window": window });
            run_report(&client, "funnel", &website_id, &start_date, &end_date, &timezone, Some(extra), json).await;
        }
        ReportsCmd::Retention {
            website_id,
            start_date,
            end_date,
            timezone,
            json,
        } => {
            run_report(&client, "retention", &website_id, &start_date, &end_date, &timezone, None, json).await;
        }
        ReportsCmd::Journey {
            website_id,
            start_date,
            end_date,
            steps,
            timezone,
            json,
        } => {
            let extra = serde_json::json!({ "steps": steps });
            run_report(&client, "journey", &website_id, &start_date, &end_date, &timezone, Some(extra), json).await;
        }
        ReportsCmd::Revenue {
            website_id,
            start_date,
            end_date,
            timezone,
            json,
        } => {
            run_report(&client, "revenue", &website_id, &start_date, &end_date, &timezone, None, json).await;
        }
        ReportsCmd::Utm {
            website_id,
            start_date,
            end_date,
            timezone,
            json,
        } => {
            run_report(&client, "utm", &website_id, &start_date, &end_date, &timezone, None, json).await;
        }
        ReportsCmd::Breakdown {
            website_id,
            start_date,
            end_date,
            timezone,
            json,
        } => {
            run_report(&client, "breakdown", &website_id, &start_date, &end_date, &timezone, None, json).await;
        }
        ReportsCmd::Goal {
            website_id,
            start_date,
            end_date,
            timezone,
            json,
        } => {
            run_report(&client, "goal", &website_id, &start_date, &end_date, &timezone, None, json).await;
        }
        ReportsCmd::Performance {
            website_id,
            start_date,
            end_date,
            timezone,
            json,
        } => {
            run_report(&client, "performance", &website_id, &start_date, &end_date, &timezone, None, json).await;
        }
    }
}

async fn run_report(
    client: &UmamiClient,
    report_type: &str,
    website_id: &str,
    start_date: &str,
    end_date: &str,
    timezone: &str,
    extra_params: Option<Value>,
    json: bool,
) {
    let mut parameters = serde_json::json!({
        "startDate": start_date,
        "endDate": end_date,
        "timezone": timezone,
    });
    if let Some(extra) = extra_params {
        if let (Some(params), Some(extra_obj)) = (parameters.as_object_mut(), extra.as_object()) {
            for (k, v) in extra_obj {
                params.insert(k.clone(), v.clone());
            }
        }
    }

    let body = serde_json::json!({
        "websiteId": website_id,
        "type": report_type,
        "parameters": parameters,
    });

    match client
        .post::<Value>(&format!("/api/reports/{report_type}"), &body)
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
