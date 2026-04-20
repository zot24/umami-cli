use clap::Subcommand;
use serde_json::Value;

use crate::api::UmamiClient;
use crate::config::Config;
use crate::output::*;

#[derive(Subcommand)]
pub enum StatsCmd {
    /// Get summary statistics for a website
    Summary {
        /// Website ID
        #[arg(long)]
        website_id: String,
        /// Start time (milliseconds epoch)
        #[arg(long)]
        start_at: i64,
        /// End time (milliseconds epoch)
        #[arg(long)]
        end_at: i64,
        /// Output raw JSON
        #[arg(long)]
        json: bool,
    },
    /// Get active visitors (last 5 minutes)
    Active {
        /// Website ID
        #[arg(long)]
        website_id: String,
    },
    /// Get pageview data over time
    Pageviews {
        /// Website ID
        #[arg(long)]
        website_id: String,
        /// Start time (milliseconds epoch)
        #[arg(long)]
        start_at: i64,
        /// End time (milliseconds epoch)
        #[arg(long)]
        end_at: i64,
        /// Time unit: minute, hour, day, month, year
        #[arg(long, default_value = "day")]
        unit: String,
        /// Timezone
        #[arg(long, default_value = "UTC")]
        timezone: String,
        /// Output raw JSON
        #[arg(long)]
        json: bool,
    },
    /// Get metrics by type
    Metrics {
        /// Website ID
        #[arg(long)]
        website_id: String,
        /// Start time (milliseconds epoch)
        #[arg(long)]
        start_at: i64,
        /// End time (milliseconds epoch)
        #[arg(long)]
        end_at: i64,
        /// Metric type (e.g. url, referrer, browser, os, device, country, event)
        #[arg(long, default_value = "url")]
        r#type: String,
        /// Output raw JSON
        #[arg(long)]
        json: bool,
    },
    /// Get data availability date range
    DateRange {
        /// Website ID
        #[arg(long)]
        website_id: String,
    },
}

pub async fn run(cmd: StatsCmd) {
    let config = Config::load();
    let client = match UmamiClient::from_config(&config) {
        Ok(c) => c,
        Err(e) => {
            print_error(&e.to_string());
            return;
        }
    };

    match cmd {
        StatsCmd::Summary {
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
                .get_with_query::<Value>(&format!("/api/websites/{website_id}/stats"), &query)
                .await
            {
                Ok(data) => {
                    if json {
                        print_json(&data);
                        return;
                    }
                    println!("Pageviews: {}", val_str(&data, "pageviews"));
                    println!("Visitors:  {}", val_str(&data, "visitors"));
                    println!("Visits:    {}", val_str(&data, "visits"));
                    println!("Bounces:   {}", val_str(&data, "bounces"));
                    println!("Totaltime: {}", val_str(&data, "totaltime"));
                }
                Err(e) => print_error(&e.to_string()),
            }
        }
        StatsCmd::Active { website_id } => {
            match client
                .get::<Value>(&format!("/api/websites/{website_id}/active"))
                .await
            {
                Ok(data) => {
                    let visitors = data
                        .get("visitors")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    println!("Active visitors: {visitors}");
                }
                Err(e) => print_error(&e.to_string()),
            }
        }
        StatsCmd::Pageviews {
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
                .get_with_query::<Value>(&format!("/api/websites/{website_id}/pageviews"), &query)
                .await
            {
                Ok(data) => {
                    if json {
                        print_json(&data);
                        return;
                    }
                    if let Some(pvs) = data.get("pageviews").and_then(|p| p.as_array()) {
                        let rows: Vec<Vec<String>> = pvs
                            .iter()
                            .map(|p| vec![val_str(p, "x"), val_str(p, "y")])
                            .collect();
                        print_table(&["TIME", "PAGEVIEWS"], &rows);
                    } else {
                        print_json(&data);
                    }
                }
                Err(e) => print_error(&e.to_string()),
            }
        }
        StatsCmd::Metrics {
            website_id,
            start_at,
            end_at,
            r#type,
            json,
        } => {
            let query = vec![
                ("startAt".into(), start_at.to_string()),
                ("endAt".into(), end_at.to_string()),
                ("type".into(), r#type),
            ];
            match client
                .get_with_query::<Value>(&format!("/api/websites/{website_id}/metrics"), &query)
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
                            .map(|m| vec![val_str(m, "x"), val_str(m, "y")])
                            .collect();
                        print_table(&["NAME", "COUNT"], &rows);
                    } else {
                        print_json(&data);
                    }
                }
                Err(e) => print_error(&e.to_string()),
            }
        }
        StatsCmd::DateRange { website_id } => {
            match client
                .get::<Value>(&format!("/api/websites/{website_id}/daterange"))
                .await
            {
                Ok(data) => print_json(&data),
                Err(e) => print_error(&e.to_string()),
            }
        }
    }
}
