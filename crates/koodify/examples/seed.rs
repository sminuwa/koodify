use anyhow::Result;
use koodify::db;

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenvy::dotenv();

    let url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/koodify".into());
    let pool = db::connect(&url).await?;
    db::migrate(&pool).await?;
    db::seed::run(&pool).await?;
    println!("seed complete for {url}");
    Ok(())
}
