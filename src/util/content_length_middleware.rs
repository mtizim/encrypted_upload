// https://stackoverflow.com/questions/71714621/actix-web-limit-upload-file-size
use actix_web::Error;
use std::{
    future::{ready, Ready},
    io::ErrorKind,
};

use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use futures_util::future::LocalBoxFuture;

pub struct ContentLengthLimit {
    pub limit: u64, // byte
}
impl<S, B> Transform<S, ServiceRequest> for ContentLengthLimit
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = ContentLengthLimitMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(ContentLengthLimitMiddleware {
            service,
            limit: self.limit,
        }))
    }
}

impl ContentLengthLimit {
    pub fn new(limit: u64) -> Self {
        Self { limit }
    }
}

pub struct ContentLengthLimitMiddleware<S> {
    service: S,
    limit: u64,
}

impl<S, B> ContentLengthLimitMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    fn is_big(&self, req: &ServiceRequest) -> Result<bool, ()> {
        let a = req
            .headers()
            .get("content-length")
            .ok_or(())?
            .to_str()
            .map_err(|_| ())?
            .parse::<u64>()
            .map_err(|_| ())?;
        Ok(a > self.limit)
    }
}

impl<S, B> Service<ServiceRequest> for ContentLengthLimitMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        if let Ok(r) = self.is_big(&req) {
            if r {
                return Box::pin(async {
                    Err(std::io::Error::new(ErrorKind::Other, "too_large").into())
                });
            }
        }

        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await?;
            Ok(res)
        })
    }
}
