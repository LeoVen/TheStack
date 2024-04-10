use the_stack::api::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    let env = the_stack::tracing::setup();
    let metrics = the_stack::metrics::setup(&env)?;
    let db = the_stack::database::setup(&env).await?;
    let cache = the_stack::cache::setup(&env).await?;
    let timeout = the_stack::jobs::worker::setup(cache.clone(), db.clone(), metrics.clone())?;

    the_stack::api::setup(
        &env,
        AppState {
            db,
            cache,
            metrics,
            timeout,
        },
    )
    .await?;

    tracing::info!("Program end");
    Ok(())
}
