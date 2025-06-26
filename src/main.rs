use axum::{Router, routing::get};
use rust_embed::RustEmbed;
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use tracing::info;
use tracing_subscriber::{fmt::format::FmtSpan, layer::SubscriberExt, util::SubscriberInitExt};

use crate::budget::views::{budget_details_handler, dashboard_handler};

pub mod budget;

// Embed static files at compile time
#[derive(RustEmbed, Clone)]
#[folder = "static/"]
struct StaticFiles;

// App state to share the database pool
#[derive(Clone, Debug)]
pub struct AppState {
    pub db: SqlitePool,
}

async fn setup_database() -> SqlitePool {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL should have been set");

    info!("Connecting to database: {}", database_url);

    // Create connection pool
    let pool = SqlitePoolOptions::new()
        .max_connections(20)
        .connect(&database_url)
        .await
        .expect("unable to create database connection pool");

    // Run embedded migrations
    info!("Running database migrations...");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("migrations should have succeeded");

    info!("Database setup complete");
    pool
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    #[cfg(debug_assertions)]
    let fmt_layer = tracing_subscriber::fmt::layer().pretty();

    #[cfg(not(debug_assertions))]
    let fmt_layer = tracing_subscriber::fmt::layer().json();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "debug".into()),
        )
        .with(
            fmt_layer
                .with_span_events(FmtSpan::CLOSE)
                .with_target(false)
                .with_thread_ids(true)
                .with_thread_names(true),
        )
        .init();

    info!("Starting server...");

    // Setup database
    let pool = setup_database().await;

    // Create app state
    let state = AppState { db: pool };

    let static_assets = axum_embed::ServeEmbed::<StaticFiles>::new();

    let app = Router::new()
        .route("/", get(dashboard_handler))
        .route("/budgets/{id}", get(budget_details_handler))
        .nest_service("/static", static_assets)
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3333").await.unwrap();

    info!("Listening on: http://localhost:3333");

    axum::serve(listener, app).await.unwrap()
}
