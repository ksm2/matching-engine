use std::convert::Infallible;
use std::net::SocketAddr;

use hyper::header::CONTENT_TYPE;
use hyper::http::HeaderValue;
use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use serde::Serialize;
use tokio::sync::mpsc::Sender;

use crate::model::OpenOrder;

#[derive(Debug, Clone)]
struct AppContext {
    tx: Sender<OpenOrder>,
}

impl AppContext {
    pub(crate) fn new(tx: Sender<OpenOrder>) -> Self {
        Self { tx }
    }
}

pub async fn api(tx: Sender<OpenOrder>) {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    let context = AppContext::new(tx);

    let make_service = make_service_fn(move |conn: &AddrStream| {
        // We have to clone the context to share it with each invocation of
        // `make_service`. If your data doesn't implement `Clone` consider using
        // an `std::sync::Arc`.
        let context = context.clone();

        // You can grab the address of the incoming connection like so.
        let addr = conn.remote_addr();
        println!("Connected {}", addr);

        // Create a `Service` for responding to the request.
        let service = service_fn(move |req| handle(context.clone(), req));

        // Return the service to hyper.
        async move { Ok::<_, Infallible>(service) }
    });

    let server = Server::bind(&addr).serve(make_service);
    println!("Server is running on http://{}", addr);
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}

async fn handle(context: AppContext, req: Request<Body>) -> Result<Response<Body>, Infallible> {
    println!("{} {}", req.method(), req.uri());
    match (req.method(), req.uri().path()) {
        (&Method::POST, "/orders") => handle_open_order(context, req.into_body()).await,
        _ => not_found(),
    }
}

async fn handle_open_order(context: AppContext, req: Body) -> Result<Response<Body>, Infallible> {
    let str = hyper::body::to_bytes(req).await.unwrap();
    let order = serde_json::from_slice(&str).unwrap();
    let res = json_response(&order);
    context.tx.send(order).await.unwrap();
    Ok(res)
}

fn json_response<T: Serialize>(data: &T) -> Response<Body> {
    let json = serde_json::to_string(data).unwrap();
    let mut res = Response::new(json.into());
    let headers = res.headers_mut();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    res
}

fn not_found() -> Result<Response<Body>, Infallible> {
    let mut res = Response::default();
    *res.status_mut() = StatusCode::NOT_FOUND;
    Ok(res)
}
