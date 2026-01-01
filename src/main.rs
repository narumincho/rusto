use std::net::SocketAddr;

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

    let app = axum::Router::new().route("/", axum::routing::get(|| async { "Hello, World!" }));

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
