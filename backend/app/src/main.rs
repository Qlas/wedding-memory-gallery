use errors::AppError;
use state::AppState;

mod database;
mod dto;
mod errors;
mod routers;
mod state;
mod thumbnail;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let state = AppState::builder().try_build().await?;
    tracing_subscriber::fmt().init();

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, routers::app(state)).await?;

    Ok(())
}
