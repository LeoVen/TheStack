use the_stack::api::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let env = the_stack::tracing::setup();
    let metrics = the_stack::metrics::setup(&env)?;
    let db = the_stack::database::setup(&env).await?;
    let jwt_service: the_stack::jwt::JWTService = the_stack::jwt::setup()?;
    let cache = the_stack::cache::setup(&env).await?;
    let timeout = the_stack::jobs::worker::setup(cache.clone(), db.clone(), metrics.clone())?;

    the_stack::api::setup(
        &env,
        AppState {
            db,
            cache,
            metrics,
            timeout,
            jwt_service,
        },
    )
    .await?;

    tracing::info!("Program end");
    Ok(())
}
