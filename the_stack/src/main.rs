#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let env = the_stack::tracing::setup();
    let metrics = the_stack::metrics::setup(&env)?;
    let batch_config = the_stack::service::BatchInsertConfig::new()?;
    let db = the_stack::database::setup(&env).await?;
    let jwt_service = the_stack::auth::jwt::setup()?;
    let (cache, lock) = the_stack::cache::setup(&env).await?;
    let timeout = the_stack::jobs::worker::setup(cache.clone(), db.clone(), metrics.clone())?;
    let user_login = the_stack::service::userlogin::UserLoginService::new(
        the_stack::database::userlogin::UserLoginRepository::new(db.clone()),
    )?;
    let user_auth =
        the_stack::auth::userlogin::UserAuthMiddleware::new(jwt_service.clone(), user_login);
    let kc_auth = the_stack::auth::keycloak::KeycloakAuthMiddleware::new()?;

    the_stack::api::setup(
        &env,
        the_stack::api::AppState {
            db,
            cache,
            metrics,
            timeout,
            jwt_service,
            lock,
            batch_config,
            user_auth,
            kc_auth,
        },
    )
    .await?;

    tracing::info!("Program end");
    Ok(())
}
