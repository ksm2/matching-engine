mod buckets;
mod context;
mod disconnect;
mod error;

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
use serde::{Deserialize, Serialize};
use tokio::signal;
use tokio::time::Instant;

pub use self::context::Context;
use self::disconnect::with_disconnect_fn;
use self::error::{to_http_err, HttpResult};
use crate::config::Config;

pub async fn api(config: Config, context: Context) {
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
        context.inc_connections();

        // Create a `Service` for responding to the request.
        let ctx = context.clone();
        let service = service_fn(move |req| handle(ctx.clone(), req));

        // Listen for the service being disconnected.
        let dropping = with_disconnect_fn(service, move || {
            debug!("Disconnected {}", addr);
            context.dec_connections();
        });

        // Return the service to hyper.
        async move { Ok::<_, Infallible>(dropping) }
    });

    // Bind server to address
    let server = Server::bind(&addr).tcp_nodelay(true).serve(make_service);

    // Listen to Ctrl C being triggered for graceful shutdown
    let graceful = server.with_graceful_shutdown(async {
        signal::ctrl_c().await.ok();
    });

    // Listen for requests
    info!("Server is running on http://{}", addr);
    if let Err(e) = graceful.await {
        error!("Server error: {}", e);
    }

    info!("Server has shut down");
}

/// Handles an incoming request
async fn handle(context: Context, req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let method = req.method().clone();
    let uri = req.uri().clone();

    let time = Instant::now();
    let res = handle_routing(&context, req)
        .await
        .unwrap_or_else(|err| err.into());
    let elapsed = time.elapsed();

    context.observe_req_duration(&method, uri.path(), elapsed);
    debug!("{} {} {} {:?}", &method, uri.path(), res.status(), elapsed);
    Ok(res)
}

async fn handle_routing(context: &Context, req: Request<Body>) -> HttpResult<Response<Body>> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => handle_get_order_book(context).await,
        (&Method::GET, "/stream") => handle_stream(context).await,
        (_other_method, "/") => method_not_allowed(&[Method::GET]),

        (&Method::GET, "/trades") => handle_get_trades(context).await,
        (_other_method, "/trades") => method_not_allowed(&[Method::GET]),

        (&Method::POST, "/orders") => handle_open_order(context, req.into_body()).await,
        (_other_method, "/orders") => method_not_allowed(&[Method::POST]),

        (&Method::GET, "/metrics") => handle_metrics(context),
        (_other_method, "/metrics") => method_not_allowed(&[Method::GET]),

        _ => not_found(),
    }
}

async fn handle_stream(context: &Context) -> HttpResult<Response<Body>> {
    let stream = futures::stream::unfold((), move |state| async move {
        let book = context.book_receiver.recv().await?;
        let json = serde_json::to_string(book).unwrap();
        let bytes = json.into();
        let result: Result<Vec<u8>, Infallible> = Ok(bytes);
        Some((result, ()))
    });

    let mut body = Body::wrap_stream(stream);
    let mut res = Response::new(body);
    Ok(res)
}

async fn handle_get_order_book(context: &Context) -> HttpResult<Response<Body>> {
    let order_book = context.read_order_book().await;
    let res = json_response(StatusCode::OK, &order_book.deref())?;
    Ok(res)
}

async fn handle_get_trades(context: &Context) -> HttpResult<Response<Body>> {
    let trades = context.read_trades().await;
    let res = json_response(StatusCode::OK, &trades.deref())?;
    Ok(res)
}

async fn handle_open_order(context: &Context, req: Body) -> HttpResult<Response<Body>> {
    let order = json_request(req).await?;
    let order = context.open_order(order).await?;
    let res = json_response(StatusCode::CREATED, &order)?;
    Ok(res)
}

async fn json_request<T: for<'a> Deserialize<'a>>(req: Body) -> HttpResult<T> {
    let str = hyper::body::to_bytes(req).await?;
    serde_json::from_slice::<T>(&str).map_err(to_http_err(error::BadRequest))
}

fn json_response<T: Serialize>(status: StatusCode, data: &T) -> HttpResult<Response<Body>> {
    let json = serde_json::to_string(data)?;
    let mut res = Response::new(json.into());
    *res.status_mut() = status;
    let headers = res.headers_mut();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    Ok(res)
}

fn handle_metrics(context: &Context) -> HttpResult<Response<Body>> {
    let mut buffer = vec![];
    let encoder = TextEncoder::new();
    let metrics = context.gather_metrics();

    encoder.encode(&metrics, &mut buffer)?;
    writeln!(&mut buffer, "# EOF")?;

    let res = Response::builder()
        .header(
            CONTENT_TYPE,
            "application/openmetrics-text; version=1.0.0; charset=utf-8",
        )
        .body(buffer.into())?;
    Ok(res)
}

/// Return a 405 Method Not Allowed response
fn method_not_allowed(allow: &[Method]) -> HttpResult<Response<Body>> {
    let mut res = Response::default();
    *res.status_mut() = StatusCode::METHOD_NOT_ALLOWED;

    let headers = res.headers_mut();
    let allow_str = allow.iter().map(|m| m.as_str()).collect::<Vec<_>>();
    headers.insert(ALLOW, allow_str.join(", ").parse()?);

    Ok(res)
}

/// Return a 404 Not Found response
fn not_found() -> HttpResult<Response<Body>> {
    let mut res = Response::default();
    *res.status_mut() = StatusCode::NOT_FOUND;
    Ok(res)
}
