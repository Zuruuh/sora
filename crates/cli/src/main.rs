// CLI usages examples
// sora-cli create-fixtures
// sora-cli show
// sora-cli show --filter="user"
// sora-cli show --filter="agr-22795DC7-E972-44D7-A74B-553EA6589044"
// sora-cli simulate --duration 2
#![feature(iter_map_windows)]

use clap::Parser;
use fixtures::create_fixtures;
use show::show;
use simulation::simulate;
use sqlx::postgres::PgPool;
use std::env;

mod fixtures;
pub mod range;
mod show;
mod simulation;

#[tokio::main]
pub async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .format_timestamp(None)
        .init();

    let pool = PgPool::connect(env::var("DATABASE_URL")?.as_str()).await?;

    let CliArguments { subcommand: args } = CliArguments::parse();
    let mut rng = rand::thread_rng();

    match args {
        Command::CreateFixtures { subdivide } => create_fixtures(&pool, &mut rng, subdivide).await,
        Command::Simulate {/* duration */} => simulate(/*duration*/24, &pool).await,
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
        /*
                /// Rental duration in months
                #[arg(long, short, default_value_t = 24)]
                duration: usize,
        */
    },
}
