use askama::Template;
use axum::{
    debug_handler,
    extract::{Path, State},
    http::StatusCode,
    response::Html,
};
use tracing::{info, instrument};

use crate::AppState;

// Askama template
#[derive(Template)]
#[template(path = "dashboard.html")]
pub struct DashboardTemplate {}

#[instrument]
pub async fn dashboard_handler(State(state): State<AppState>) -> Result<Html<String>, StatusCode> {
    let template = DashboardTemplate {};

    match template.render() {
        Ok(html) => {
            info!("Dashboard template rendered successfully");
            Ok(Html(html))
        }
        Err(e) => {
            tracing::error!("Failed to render template: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[debug_handler]
pub async fn budget_details_handler(Path(id): Path<i64>) -> Result<Html<String>, StatusCode> {
    _ = id;
    todo!();
}
