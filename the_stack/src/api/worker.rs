use std::sync::Arc;
use std::sync::Mutex;

use axum::extract::Path;
use axum::extract::State;
use axum::Router;

use crate::api::AppState;
use crate::error::api::ApiResult;

struct WorkerAppState {
    timeout: Arc<Mutex<u64>>,
}

pub fn router(ctx: AppState) -> Router {
    Router::<Arc<WorkerAppState>>::new()
        .route(
            "/worker/timeout_seconds/:timeout",
            axum::routing::put(set_timeout),
        )
        .with_state(
            (WorkerAppState {
                timeout: ctx.timeout,
            })
            .into(),
        )
}

#[tracing::instrument(skip_all)]
async fn set_timeout(
    State(ctx): State<Arc<WorkerAppState>>,
    Path(timeout): Path<u64>,
) -> ApiResult<()> {
    let mut lock = ctx.timeout.lock().expect("mutext lock PoisonError");

    *lock = timeout;

    Ok(())
}
