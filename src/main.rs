use hyper::service::{make_service_fn, Service};
use hyper::{Body, Request, Response, Server};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::{convert::Infallible, net::SocketAddr};

#[tokio::main]
async fn main() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    let make_svc = make_service_fn(|_conn| async { Ok::<_, Infallible>(Svc { count: 0 }) });

    let server = Server::bind(&addr).serve(make_svc);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}

struct Svc {
    count: u32,
}

impl Service<Request<Body>> for Svc {
    type Response = Response<Body>;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        println!("I am being called for {:?}", req);
        self.count += 1;
        let msg = format!("{}, {}!", req.method(), self.count);
        let res = Ok(Response::new(msg.into()));
        Box::pin(async { res })
    }
}
