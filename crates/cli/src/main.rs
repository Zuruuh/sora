// CLI usages examples
// sora-cli create-fixtures
// sora-cli show
// sora-cli show --filter="user"
// sora-cli show --filter="agr-22795DC7-E972-44D7-A74B-553EA6589044"
// sora-cli simulate ofc-22795DC7-E972-44D7-A74B-553EA6589044

use clap::Parser;
use sqlx::postgres::PgPool;
use std::env;

#[tokio::main]
pub async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let pool = PgPool::connect(env::var("DATABASE_URL")?.as_str()).await?;

    let value = sqlx::query!("SELECT 1 as my_selected_value")
        .fetch_one(&pool)
        .await
        .map(|rec| rec.my_selected_value)
        .unwrap_or_default()
        .unwrap_or_default();

    println!("{value}");

    let CliArguments { subcommand: args } = CliArguments::parse();

    match args {
        Command::CreateFixtures => {}
        Command::Simulate { duration } => {}
        Command::Show { filter } => {}
    };

    Ok(())
}

#[derive(clap::Parser, Debug)]
#[command(version)]
pub struct CliArguments {
    #[command(subcommand)]
    subcommand: Command,
}

#[derive(clap::Subcommand, Debug)]
pub enum Command {
    /// Create in-database fixtures
    CreateFixtures,
    /// View one or multiple entities
    Show {
        #[arg(long, short)]
        // TODO use a custom enum instead of an Option<String> here
        filter: Option<String>,
    },
    /// Simulate an office rental
    Simulate {
        /// Rental duration in months
        #[arg(long, short, default_value_t = 24)]
        duration: usize,
    },
}
