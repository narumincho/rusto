use std::net::SocketAddr;

use axum::{body, http::request};

async fn get_notion_api_key() -> String {
    match std::env::var("NOTION_KEY") {
        // in Cloud Run
        Ok(val) => val,
        // local dev
        Err(_) => std::fs::read_to_string("./notionApiKey.txt").unwrap(),
    }
}

#[tokio::main]
pub async fn main() {
    let addr = match std::env::var("PORT") {
        // in Cloud Run
        Ok(port) => SocketAddr::from(([0, 0, 0, 0], port.parse().expect("PORT must be a number"))),
        // local dev
        Err(_) => SocketAddr::from(([127, 0, 0, 1], 3000)),
    };

    println!("Listening on http://{}", addr);

    let app = axum::Router::new().route("/", axum::routing::get(get_handler).post(post_handler));

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn get_handler() -> String {
    "Hello, World!".to_string()
}

async fn post_handler(request: axum::extract::Request) -> String {
    let body_bytes = axum::body::to_bytes(request.into_body(), usize::MAX)
        .await
        .unwrap();
    // Bytes を String に変換（UTF-8 として扱う）
    let body_as_string = String::from_utf8(body_bytes.to_vec()).unwrap();
    // 例: String を表示または返す
    println!("Body as String: {}", body_as_string);
    body_as_string
}
