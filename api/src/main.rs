use axum::{
    routing::{delete, get, post, put},
    Router,
};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tracing_subscriber::EnvFilter;

mod auth;
mod error;
mod routes;

pub use auth::JwksVerifier;

#[derive(Clone)]
pub struct AppState {
    pub pool:        sqlx::PgPool,
    pub verifier:    Arc<JwksVerifier>,
    pub prov_secret: String,
}

// Allow axum extractors to pull the verifier out of AppState.
impl axum::extract::FromRef<AppState> for Arc<JwksVerifier> {
    fn from_ref(state: &AppState) -> Self {
        state.verifier.clone()
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    let database_url = std::env::var("DATABASE_URL")?;
    let jwks_url     = std::env::var("JWKS_URL")?;
    let prov_secret  = std::env::var("PROVISIONING_SECRET").unwrap_or_default();

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;

    // Run embedded SQL migrations on startup.
    sqlx::migrate!("./migrations").run(&pool).await?;
    tracing::info!("Migrations applied");

    let state = AppState {
        pool,
        verifier: Arc::new(JwksVerifier::new(jwks_url)),
        prov_secret,
    };

    let app = Router::new()
        .route("/health", get(routes::health))
        // Notes
        .route("/notes",     get(routes::notes::list).post(routes::notes::create))
        .route("/notes/{id}", get(routes::notes::get_one)
                                .put(routes::notes::update)
                                .delete(routes::notes::delete))
        // Todos
        .route("/notes/{id}/todos",           post(routes::todos::create))
        .route("/notes/{id}/todos/{todo_id}", put(routes::todos::update)
                                               .delete(routes::todos::delete))
        // Sharing
        .route("/notes/{id}/share",              post(routes::shares::share))
        .route("/notes/{id}/share/{share_id}",   delete(routes::shares::unshare))
        // Admin (internal only — not exposed through nginx)
        .route("/admin/provision-user",          post(routes::admin::provision))
        .route("/admin/provision-user/{user_id}", delete(routes::admin::deprovision))
        .with_state(state)
        .layer(CorsLayer::permissive());

    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    tracing::info!("Listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;
    Ok(())
}
