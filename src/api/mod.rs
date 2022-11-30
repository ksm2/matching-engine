use std::convert::Infallible;
use std::io::Write;
use std::ops::Deref;

use hyper::header::{ALLOW, CONTENT_TYPE};
use hyper::http::HeaderValue;
use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use log::{debug, error, info};
use prometheus::{Encoder, TextEncoder};
use serde::Serialize;
use tokio::time::Instant;

use crate::config::Config;
use crate::model::ApiContext;

pub async fn api(config: Config, context: ApiContext) {
    let Ok(addr) = config.host.parse() else {
        error!("Could not parse APP_HOST: {}", config.host);
        return;
    };

    let make_service = make_service_fn(move |conn: &AddrStream| {
        // We have to clone the context to share it with each invocation of
        // `make_service`. If your data doesn't implement `Clone` consider using
        // an `std::sync::Arc`.
        let context = context.clone();

        // You can grab the address of the incoming connection like so.
        let addr = conn.remote_addr();
        debug!("Connected {}", addr);

        // Create a `Service` for responding to the request.
        let service = service_fn(move |req| handle(context.clone(), req));

        // Return the service to hyper.
        async move { Ok::<_, Infallible>(service) }
    });

    let server = Server::bind(&addr).serve(make_service);
    info!("Server is running on http://{}", addr);
    if let Err(e) = server.await {
        error!("Server error: {}", e);
    }
}

async fn handle(context: ApiContext, req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let time = Instant::now();
    let method = req.method().clone();
    let uri = req.uri().clone();
    let res = match (&method, uri.path()) {
        (&Method::GET, "/") => handle_get_order_book(&context).await,
        (_, "/") => method_not_allowed(&[Method::GET]),

        (&Method::GET, "/trades") => handle_get_trades(&context).await,
        (_, "/trades") => method_not_allowed(&[Method::GET]),

        (&Method::POST, "/orders") => handle_open_order(&context, req.into_body()).await,
        (_, "/orders") => method_not_allowed(&[Method::POST]),

        (&Method::GET, "/metrics") => handle_metrics(&context),
        (_, "/metrics") => method_not_allowed(&[Method::GET]),

        _ => not_found(),
    }?;
    let elapsed = time.elapsed();
    context.observe_req_duration(&method, uri.path(), elapsed);
    debug!("{} {} {} {:?}", &method, uri.path(), res.status(), elapsed);
    Ok(res)
}

async fn handle_get_order_book(context: &ApiContext) -> Result<Response<Body>, Infallible> {
    let order_book = context.read_order_book().await;
    let res = json_response(StatusCode::OK, &order_book.deref());
    Ok(res)
}

async fn handle_get_trades(context: &ApiContext) -> Result<Response<Body>, Infallible> {
    let trades = context.read_trades().await;
    let res = json_response(StatusCode::OK, &trades.deref());
    Ok(res)
}

async fn handle_open_order(context: &ApiContext, req: Body) -> Result<Response<Body>, Infallible> {
    let str = hyper::body::to_bytes(req).await.unwrap();
    let order = serde_json::from_slice(&str).unwrap();
    let order = context.open_order(order).await.unwrap();
    let res = json_response(StatusCode::CREATED, &order);
    Ok(res)
}

fn json_response<T: Serialize>(status: StatusCode, data: &T) -> Response<Body> {
    let json = serde_json::to_string(data).unwrap();
    let mut res = Response::new(json.into());
    *res.status_mut() = status;
    let headers = res.headers_mut();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    res
}

fn handle_metrics(context: &ApiContext) -> Result<Response<Body>, Infallible> {
    let mut buffer = vec![];
    let encoder = TextEncoder::new();
    let metrics = context.gather_metrics();

    encoder.encode(&metrics, &mut buffer).unwrap();
    writeln!(&mut buffer, "# EOF").unwrap();

    let res = Response::builder()
        .header(
            CONTENT_TYPE,
            "application/openmetrics-text; version=1.0.0; charset=utf-8",
        )
        .body(buffer.into())
        .unwrap();
    Ok(res)
}

fn method_not_allowed(allow: &[Method]) -> Result<Response<Body>, Infallible> {
    let mut res = Response::default();
    *res.status_mut() = StatusCode::METHOD_NOT_ALLOWED;

    let headers = res.headers_mut();
    let allow_str = allow
        .iter()
        .map(|m| m.as_str())
        .intersperse(", ")
        .collect::<String>();
    headers.insert(ALLOW, allow_str.parse().unwrap());

    Ok(res)
}

fn not_found() -> Result<Response<Body>, Infallible> {
    let mut res = Response::default();
    *res.status_mut() = StatusCode::NOT_FOUND;
    Ok(res)
}
