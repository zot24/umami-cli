mod api;
mod commands;
mod config;
mod output;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "umami-cli")]
#[command(about = "CLI tool for managing self-hosted Umami analytics instances")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Authentication (login, logout, verify)
    Auth {
        #[command(subcommand)]
        cmd: commands::auth::AuthCmd,
    },
    /// Manage websites
    Websites {
        #[command(subcommand)]
        cmd: commands::websites::WebsitesCmd,
    },
    /// View website statistics
    Stats {
        #[command(subcommand)]
        cmd: commands::stats::StatsCmd,
    },
    /// Manage and track events
    Events {
        #[command(subcommand)]
        cmd: commands::events::EventsCmd,
    },
    /// View session data
    Sessions {
        #[command(subcommand)]
        cmd: commands::sessions::SessionsCmd,
    },
    /// Run and manage reports
    Reports {
        #[command(subcommand)]
        cmd: commands::reports::ReportsCmd,
    },
    /// View realtime analytics
    Realtime {
        #[command(subcommand)]
        cmd: commands::realtime::RealtimeCmd,
    },
    /// Manage teams
    Teams {
        #[command(subcommand)]
        cmd: commands::teams::TeamsCmd,
    },
    /// User management
    Users {
        #[command(subcommand)]
        cmd: commands::users::UsersCmd,
    },
    /// Admin operations (self-hosted only)
    Admin {
        #[command(subcommand)]
        cmd: commands::admin::AdminCmd,
    },
    /// Manage share pages
    Shares {
        #[command(subcommand)]
        cmd: commands::shares::SharesCmd,
    },
    /// Manage tracked links
    Links {
        #[command(subcommand)]
        cmd: commands::links::LinksCmd,
    },
    /// Manage tracking pixels
    Pixels {
        #[command(subcommand)]
        cmd: commands::pixels::PixelsCmd,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Auth { cmd } => commands::auth::run(cmd).await,
        Commands::Websites { cmd } => commands::websites::run(cmd).await,
        Commands::Stats { cmd } => commands::stats::run(cmd).await,
        Commands::Events { cmd } => commands::events::run(cmd).await,
        Commands::Sessions { cmd } => commands::sessions::run(cmd).await,
        Commands::Reports { cmd } => commands::reports::run(cmd).await,
        Commands::Realtime { cmd } => commands::realtime::run(cmd).await,
        Commands::Teams { cmd } => commands::teams::run(cmd).await,
        Commands::Users { cmd } => commands::users::run(cmd).await,
        Commands::Admin { cmd } => commands::admin::run(cmd).await,
        Commands::Shares { cmd } => commands::shares::run(cmd).await,
        Commands::Links { cmd } => commands::links::run(cmd).await,
        Commands::Pixels { cmd } => commands::pixels::run(cmd).await,
    }
}
