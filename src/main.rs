mod notion;
mod notion_minecraft_db;

use std::net::SocketAddr;

#[tokio::main]
pub async fn main() {
    let addr = match std::env::var("PORT") {
        // in Cloud Run
        Ok(port) => SocketAddr::from(([0, 0, 0, 0], port.parse().expect("PORT must be a number"))),
        // local dev
        Err(_) => SocketAddr::from(([127, 0, 0, 1], 3000)),
    };

    println!("Listening on http://{}", addr);

    let app = axum::Router::new().route(
        "/",
        axum::routing::get(get_handler).post(notion_minecraft_db::post_handler),
    );

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn get_handler() -> String {
    "Hello, World!".to_string()
}
