use anyhow::Context;
use tracing_subscriber::EnvFilter;
use koodify::build_app;
use koodify::config::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load `.env` from CWD when present. Does not override vars already set
    // by systemd / docker / the shell — those win.
    let _ = dotenvy::dotenv();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    let config = Config::from_env()?;
    let bind = config.bind.clone();
    let (app, state) = build_app(config).await?;
    koodify::services::reconcile::spawn_loop(state);

    let listener = tokio::net::TcpListener::bind(&bind)
        .await
        .with_context(|| format!("failed to bind {bind}"))?;

    tracing::info!(%bind, "koodify listening");
    axum::serve(listener, app).await?;
    Ok(())
}
