use axum::routing::get;

use crate::handlers;

use super::AppState;

pub fn add_routers(router: axum::Router<AppState>) -> axum::Router<AppState> {
    router.route("/api/health_check", get(handlers::server::health_check))
}
