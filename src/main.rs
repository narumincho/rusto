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

    let app = axum::Router::new()
        .route(
            "/",
            axum::routing::get(get_handler).post(notion_minecraft_db::post_handler),
        )
        .route("/frontend.js", axum::routing::get(get_frontend_js))
        .route("/frontend_bg.wasm", axum::routing::get(get_frontend_wasm));

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn get_frontend_js() -> axum::response::Response<String> {
    let js_code = std::fs::read_to_string("frontend/pkg/frontend.js").unwrap();
    axum::response::Response::builder()
        .status(200)
        .header("content-type", "application/javascript")
        .body(js_code)
        .unwrap()
}

async fn get_frontend_wasm() -> (
    axum::http::StatusCode,
    axum::http::header::HeaderMap,
    Vec<u8>,
) {
    let wasm = std::fs::read("frontend/pkg/frontend_bg.wasm").unwrap();
    let mut headers = axum::http::header::HeaderMap::new();
    headers.insert("content-type", "application/wasm".parse().unwrap());
    (axum::http::StatusCode::OK, headers, wasm)
}

async fn get_handler() -> axum::response::Html<String> {
    let now = chrono::Local::now();
    let html = format!(
        r#"
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <title>Current Time</title>
            <style>
                body {{
                    font-family: sans-serif;
                    display: flex;
                    justify_content: center;
                    align_items: center;
                    height: 100vh;
                    background-color: #f0f0f0;
                    margin: 0;
                    flex-direction: column;
                }}
                .container {{
                    text-align: center;
                    background: white;
                    padding: 2rem;
                    border-radius: 10px;
                    box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
                }}
                h1 {{
                    color: #333;
                }}
                p {{
                    font-size: 1.5rem;
                    color: #666;
                }}
            </style>
        </head>
        <body>
            <div class="container">
                <h1>Current Local Time</h1>
                <p>{}</p>
            </div>
            <script type="module">
                import init, {{ greet }} from './frontend.js';
                async function run() {{
                    await init();
                    greet();
                }}
                run();
            </script>
        </body>
        </html>
        "#,
        now.format("%Y-%m-%d %H:%M:%S")
    );
    axum::response::Html(html)
}
