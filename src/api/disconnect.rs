use hyper::service::Service;
use std::marker::PhantomData;
use std::task::{Context, Poll};

pub struct DisconnectService<T, F, R>
where
    T: Service<R>,
    F: FnMut(),
{
    service: T,
    disconnect: F,
    request: PhantomData<R>,
}

impl<T, F, R> DisconnectService<T, F, R>
where
    T: Service<R>,
    F: FnMut(),
{
    fn new(service: T, disconnect: F) -> Self {
        Self {
            service,
            disconnect,
            request: PhantomData,
        }
    }
}

impl<T, F, R> Drop for DisconnectService<T, F, R>
where
    T: Service<R>,
    F: FnMut(),
{
    fn drop(&mut self) {
        (self.disconnect)();
    }
}

impl<T, F, R> Service<R> for DisconnectService<T, F, R>
where
    T: Service<R>,
    F: FnMut(),
{
    type Response = T::Response;
    type Error = T::Error;
    type Future = T::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, req: R) -> Self::Future {
        self.service.call(req)
    }
}

pub fn with_disconnect_fn<T, F, R>(service: T, disconnect: F) -> DisconnectService<T, F, R>
where
    T: Service<R>,
    F: FnMut(),
{
    DisconnectService::new(service, disconnect)
}
