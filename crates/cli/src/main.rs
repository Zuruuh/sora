// CLI usages examples
// sora-cli create-fixtures
// sora-cli show
// sora-cli show --filter="user"
// sora-cli show --filter="agr-22795DC7-E972-44D7-A74B-553EA6589044"
// sora-cli simulate --duration 2

use clap::Parser;
use fixtures::create_fixtures;
use show::show;
use sqlx::postgres::PgPool;
use std::env;

mod fixtures;
mod show;

#[tokio::main]
pub async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    let pool = PgPool::connect(env::var("DATABASE_URL")?.as_str()).await?;

    // let value = sqlx::query!("SELECT 1 as my_selected_value")
    //     .fetch_one(&pool)
    //     .await
    //     .map(|rec| rec.my_selected_value)
    //     .unwrap_or_default()
    //     .unwrap_or_default();
    // println!("{value}");

    let CliArguments { subcommand: args } = CliArguments::parse();
    let mut rng = rand::thread_rng();

    match args {
        Command::CreateFixtures { subdivide } => create_fixtures(&pool, &mut rng, subdivide).await,
        Command::Simulate { duration } => Ok(()),
        Command::Show { filter } => show(&pool, filter).await,
    }
}

#[derive(clap::Parser, Debug)]
#[command(version)]
pub struct CliArguments {
    #[command(subcommand)]
    subcommand: Command,
}

#[derive(clap::Subcommand, Debug)]
pub enum Command {
    /// Create in-database fixtures; Will truncate existing data
    CreateFixtures {
        /// Whether or not to subdivide generated offices
        #[arg(long, short)]
        subdivide: bool,
    },
    /// View one or multiple entities
    Show {
        /// Select which data should be displayed. Can be an ID prefix ("usr", "ofc"), or a table
        /// name ("users", "offices")
        #[arg(long, short)]
        filter: Option<String>,
    },
    /// Simulate an office rental
    Simulate {
        /// Rental duration in months
        #[arg(long, short, default_value_t = 24)]
        duration: usize,
    },
}
