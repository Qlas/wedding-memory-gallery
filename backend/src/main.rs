use std::path::PathBuf;

mod errors;
mod routers;

#[tokio::main]
async fn main() {
    let storage_directory = PathBuf::from("storage/");
    tracing_subscriber::fmt().init();

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, routers::app(storage_directory))
        .await
        .unwrap();
}
