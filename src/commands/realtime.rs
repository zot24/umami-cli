use clap::Subcommand;
use serde_json::Value;

use crate::api::UmamiClient;
use crate::config::Config;
use crate::output::*;

#[derive(Subcommand)]
pub enum RealtimeCmd {
    /// Get realtime data for a website (last 30 minutes)
    Get {
        /// Website ID
        website_id: String,
        /// Output raw JSON
        #[arg(long)]
        json: bool,
    },
}

pub async fn run(cmd: RealtimeCmd) {
    let config = Config::load();
    let client = match UmamiClient::from_config(&config) {
        Ok(c) => c,
        Err(e) => {
            print_error(&e.to_string());
            return;
        }
    };

    match cmd {
        RealtimeCmd::Get { website_id, json } => {
            match client
                .get::<Value>(&format!("/api/realtime/{website_id}"))
                .await
            {
                Ok(data) => {
                    if json {
                        print_json(&data);
                        return;
                    }

                    // Totals
                    if let Some(totals) = data.get("totals") {
                        println!("=== Realtime (last 30 min) ===");
                        println!("Views:     {}", val_str(totals, "views"));
                        println!("Visitors:  {}", val_str(totals, "visitors"));
                        println!("Events:    {}", val_str(totals, "events"));
                        println!("Countries: {}", val_str(totals, "countries"));
                    }

                    // Top URLs
                    if let Some(urls) = data.get("urls").and_then(|u| u.as_object()) {
                        println!("\n=== Top URLs ===");
                        let mut sorted: Vec<_> = urls.iter().collect();
                        sorted.sort_by(|a, b| {
                            b.1.as_u64().unwrap_or(0).cmp(&a.1.as_u64().unwrap_or(0))
                        });
                        for (url, count) in sorted.iter().take(10) {
                            println!("  {count:>5}  {url}");
                        }
                    }

                    // Top Countries
                    if let Some(countries) = data.get("countries").and_then(|c| c.as_object()) {
                        println!("\n=== Top Countries ===");
                        let mut sorted: Vec<_> = countries.iter().collect();
                        sorted.sort_by(|a, b| {
                            b.1.as_u64().unwrap_or(0).cmp(&a.1.as_u64().unwrap_or(0))
                        });
                        for (country, count) in sorted.iter().take(10) {
                            println!("  {count:>5}  {country}");
                        }
                    }

                    // Top Referrers
                    if let Some(referrers) = data.get("referrers").and_then(|r| r.as_object()) {
                        println!("\n=== Top Referrers ===");
                        let mut sorted: Vec<_> = referrers.iter().collect();
                        sorted.sort_by(|a, b| {
                            b.1.as_u64().unwrap_or(0).cmp(&a.1.as_u64().unwrap_or(0))
                        });
                        for (referrer, count) in sorted.iter().take(10) {
                            println!("  {count:>5}  {referrer}");
                        }
                    }
                }
                Err(e) => print_error(&e.to_string()),
            }
        }
    }
}
