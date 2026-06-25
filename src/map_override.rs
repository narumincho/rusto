use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::LazyLock;

use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper::header::{
    ACCEPT_ENCODING, CONNECTION, CONTENT_ENCODING, CONTENT_LENGTH, ETAG, HOST, HeaderMap,
    HeaderValue, TRANSFER_ENCODING,
};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use serde_json::Value;
use serde_json::json;
use tokio::net::TcpListener;

const LISTEN_ADDR: ([u8; 4], u16) = ([127, 0, 0, 1], 3000);
const UPSTREAM_ORIGIN: &str = "https://seikatsumain.map.morino.party";
const MARKERS_JSON_PATH: &str = "/tiles/minecraft_overworld/markers.json";
const CIRCLE_CENTER_X: i32 = 1722;
const CIRCLE_CENTER_Z: i32 = -5105;
const CIRCLE_SOURCE_HYPOTENUSE: f64 = 96.0;
const CIRCLE_SOURCE_LEG: f64 = 25.0;
static CIRCLE_RADIUS: LazyLock<f64> = LazyLock::new(|| {
    (CIRCLE_SOURCE_HYPOTENUSE * CIRCLE_SOURCE_HYPOTENUSE - CIRCLE_SOURCE_LEG * CIRCLE_SOURCE_LEG)
        .sqrt()
});
static CIRCLE_DIAMETER: LazyLock<f64> = LazyLock::new(|| *CIRCLE_RADIUS * 2.0);
static CIRCLE_OFFSET: LazyLock<f64> = LazyLock::new(|| std::f64::consts::SQRT_2 * *CIRCLE_RADIUS);

#[derive(Clone)]
struct ProxyState {
    client: reqwest::Client,
}

pub async fn server() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(LISTEN_ADDR);
    let listener = TcpListener::bind(addr).await?;
    let state = ProxyState {
        client: reqwest::Client::new(),
    };

    println!("Listening on http://{}", addr);

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let state = state.clone();

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(
                    io,
                    service_fn(move |request| {
                        let state = state.clone();
                        async move { Ok::<_, Infallible>(handle_request(request, state).await) }
                    }),
                )
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}

async fn handle_request(
    request: Request<hyper::body::Incoming>,
    state: ProxyState,
) -> Response<Full<Bytes>> {
    match proxy_request(request, state).await {
        Ok(response) => response,
        Err(err) => {
            eprintln!("Error proxying request: {:?}", err);
            response_with_status(StatusCode::BAD_GATEWAY, Bytes::from("Bad Gateway"))
        }
    }
}

async fn proxy_request(
    request: Request<hyper::body::Incoming>,
    state: ProxyState,
) -> anyhow::Result<Response<Full<Bytes>>> {
    let method = request.method().clone();
    let path_and_query = request
        .uri()
        .path_and_query()
        .map(|path_and_query| path_and_query.as_str())
        .unwrap_or("/");
    let upstream_url = format!("{}{}", UPSTREAM_ORIGIN, path_and_query);
    let is_markers_json = request.uri().path() == MARKERS_JSON_PATH;

    let (parts, body) = request.into_parts();
    let body = body.collect().await?.to_bytes();

    let mut upstream_request = state.client.request(method, upstream_url).body(body);
    for (name, value) in parts.headers.iter() {
        if should_forward_request_header(name) {
            upstream_request = upstream_request.header(name, value);
        }
    }

    let upstream_response = upstream_request.send().await?;
    let status = upstream_response.status();
    let headers = upstream_response.headers().clone();
    let body = upstream_response.bytes().await?;
    let body = if status.is_success() && is_markers_json {
        process_markers_json(&body)?
    } else {
        body
    };

    let mut response = Response::builder().status(status);
    copy_response_headers(
        response
            .headers_mut()
            .expect("response builder has headers"),
        &headers,
        is_markers_json,
    );
    response = response.header(CONTENT_LENGTH, body.len().to_string());

    Ok(response.body(Full::new(body))?)
}

fn process_markers_json(body: &Bytes) -> anyhow::Result<Bytes> {
    let mut marker_sets: Vec<Value> = serde_json::from_slice(body)?;
    marker_sets.retain(|marker_set| {
        marker_set
            .get("id")
            .and_then(Value::as_str)
            .is_none_or(|id| id != "griefprevention")
    });
    marker_sets.push(circle_marker_set());
    Ok(Bytes::from(serde_json::to_vec(&marker_sets)?))
}

fn circle_marker_set() -> Value {
    let base = (CIRCLE_SOURCE_HYPOTENUSE * CIRCLE_SOURCE_HYPOTENUSE
        - CIRCLE_SOURCE_LEG * CIRCLE_SOURCE_LEG)
        .sqrt();
    let a: i32 = (base * 3_f64.sqrt()) as i32;
    let b: i32 = (base * 3.0 / 2.0) as i32;
    let c: i32 = (base * 2_f64.sqrt()) as i32;
    println!(
        "CIRCLE_RADIUS: {}, a: {}, b: {}, c: {}",
        *CIRCLE_RADIUS, a, b, c
    );

    let markers = [-1.5, -0.5, 0.5, 1.5]
        .iter()
        .flat_map(|&x| {
            [-1.5, -0.5, 0.5, 1.5].iter().map(move |&z| {
                let x = (x * c as f64) as i32;
                let z = (z * c as f64) as i32;
                (x, z)
            })
        })
        .collect::<Vec<(i32, i32)>>();
    let absolute_markers = markers
        .into_iter()
        .map(|(x, z)| (x + CIRCLE_CENTER_X, z + CIRCLE_CENTER_Z))
        .collect::<Vec<(i32, i32)>>();
    println!(
        "{:}",
        absolute_markers
            .iter()
            .enumerate()
            .map(|(i, (x, z))| {
                // format!("- [ ] {}, 40, {}\n", x, z)
                let name = (b'A' + i as u8) as char;
                format!(
                    "waypoint:{}:{}:{}:40:{}:3:false:0:gui.xaero_default:false:0:0:false\n",
                    name, name, x, z
                )
            })
            .collect::<String>()
    );

    let markers_json = absolute_markers
        .into_iter()
        .map(move |(x, z)| {
            json!({
                "color": "#ff00ff",
                "fillColor": "#ff00ff",
                "popup": "追加領域",
                "center": {
                    "x": x ,
                    "z": z as f64,
                },
                "type": "circle",
                "radius": *CIRCLE_DIAMETER / 2.0,
            })
        })
        .collect::<Vec<Value>>();

    json!({
        "hide": false,
        "z_index": 100,
        "name": "追加領域",
        "control": true,
        "id": "rusto-added-circles",
        "markers": markers_json,
        "order": 100,
    })
}

fn should_forward_request_header(name: &hyper::header::HeaderName) -> bool {
    !matches!(
        *name,
        HOST | CONNECTION | CONTENT_LENGTH | TRANSFER_ENCODING | ACCEPT_ENCODING
    )
}

fn copy_response_headers(
    to: &mut HeaderMap<HeaderValue>,
    from: &HeaderMap<HeaderValue>,
    is_processed_body: bool,
) {
    for (name, value) in from.iter() {
        if matches!(*name, CONNECTION | CONTENT_LENGTH | TRANSFER_ENCODING)
            || (is_processed_body && matches!(*name, CONTENT_ENCODING | ETAG))
        {
            continue;
        }
        to.insert(name, value.clone());
    }
}

fn response_with_status(status: StatusCode, body: Bytes) -> Response<Full<Bytes>> {
    Response::builder()
        .status(status)
        .body(Full::new(body))
        .expect("static response is valid")
}
