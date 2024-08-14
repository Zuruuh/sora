// CLI usages examples
// sora-cli create-fixtures
// sora-cli show
// sora-cli show --filter="user"
// sora-cli show --filter="agr-22795DC7-E972-44D7-A74B-553EA6589044"
// sora-cli simulate ofc-22795DC7-E972-44D7-A74B-553EA6589044

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

    Ok(())
}
