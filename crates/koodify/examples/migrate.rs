
use anyhow::Result;
use koodify::db;

#[tokio::main]
async fn main() -> Result<()> {
    let url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/koodify".into());
    let pool = db::connect(&url).await?;
    db::migrate(&pool).await?;
    println!("migrations applied to {url}");
    Ok(())
}
