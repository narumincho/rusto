use std::convert::Infallible;
use std::net::SocketAddr;

use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

async fn handler(_: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    Ok(Response::new(Full::new(Bytes::from("Hello, World!"))))
}

async fn get_notion_api_key() -> String {
    match std::env::var("NOTION_KEY") {
        // in Cloud Run
        Ok(val) => val,
        // local dev
        Err(_) => std::fs::read_to_string("./notionApiKey.txt").unwrap(),
    }
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!(
        "Starting server... notion api key {}",
        get_notion_api_key().await
    );
    let addr = match std::env::var("PORT") {
        // in Cloud Run
        Ok(port) => SocketAddr::from(([0, 0, 0, 0], port.parse().expect("PORT must be a number"))),
        // local dev
        Err(_) => SocketAddr::from(([127, 0, 0, 1], 3000)),
    };

    println!("Listening on http://{}", addr);

    let listener = TcpListener::bind(addr).await?;

    loop {
        let (stream, _) = listener.accept().await?;

        let io = TokioIo::new(stream);

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(handler))
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}
