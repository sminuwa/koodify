use anyhow::Context;
use tracing_subscriber::EnvFilter;
use koodify::build_app;
use koodify::config::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    let config = Config::from_env()?;
    let bind = config.bind.clone();
    let (app, _state) = build_app(config).await?;

    let listener = tokio::net::TcpListener::bind(&bind)
        .await
        .with_context(|| format!("failed to bind {bind}"))?;

    tracing::info!(%bind, "koodify listening");
    axum::serve(listener, app).await?;
    Ok(())
}
