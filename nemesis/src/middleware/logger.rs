use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    web, Error, HttpMessage, http::header,
};
use futures::{
    future::{ok, LocalBoxFuture, Ready},
    StreamExt, stream,
};
use std::{
    rc::Rc,
    time::Instant,
    pin::Pin,
};
use tracing::{debug, info, warn};
use uuid::Uuid;
use bytes::Bytes;
use actix_http::body::MessageBody;
use actix_web::dev::Payload;
use futures::stream::{Stream, once};
use actix_http::Payload as ActixPayload;
use crate::codec::binary_protocol::BinaryProtocol;

pub struct RequestLogger;

impl RequestLogger {
    pub fn new() -> Self {
        RequestLogger
    }
}

impl<S, B> Transform<S, ServiceRequest> for RequestLogger
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = RequestLoggerMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(RequestLoggerMiddleware {
            service: Rc::new(service),
        })
    }
}

pub struct RequestLoggerMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for RequestLoggerMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let svc = self.service.clone();
        let request_id = Uuid::new_v4().to_string();
        let start_time = Instant::now();
        
        let method = req.method().to_string();
        let path = req.path().to_string();
        let query = req.query_string().to_string();
        let version = format!("{:?}", req.version());
        let remote_addr = req.connection_info().peer_addr().unwrap_or("unknown").to_string();
        
        info!(
            request_id = %request_id,
            method = %method,
            path = %path,
            query = %query,
            version = %version,
            remote_addr = %remote_addr,
            "Request received"
        );
        
        let mut payload = req.take_payload();
        Box::pin(async move {
            let mut body = web::BytesMut::new();
            while let Some(chunk) = payload.next().await {
                if let Ok(chunk) = chunk {
                    body.extend_from_slice(&chunk);
                }
            }
            // IF body is not empty and method is not GET, decode the body
            if !body.is_empty() && method != "GET" {
                match decode_binary_body(&body) {
                    Ok(parsed_body) => {
                        info!(request_id = %request_id, body = %parsed_body, "Parsed request body");
                    }
                    Err(err) => {
                        debug!(request_id = %request_id, error = %err, "Failed to parse binary request body");
                    }
                }
            }
            
            let payload_stream = once(async move { Ok::<Bytes, actix_web::error::PayloadError>(Bytes::from(body)) });
            let boxed_stream: Pin<Box<dyn Stream<Item = Result<Bytes, actix_web::error::PayloadError>>>> = Box::pin(payload_stream);
            req.set_payload(ActixPayload::from(boxed_stream));
            
            let res = svc.call(req).await?;
            log_response(&res, request_id, start_time);
            Ok(res)
        })
    }
}

fn decode_binary_body(bytes: &web::BytesMut) -> Result<String, anyhow::Error> {
    let mut protocol = BinaryProtocol::new();
    protocol.set_buffer(bytes.to_vec());
    let value = protocol.decode_to_json()?;
    Ok(serde_json::to_string_pretty(&value)?)
}

fn log_response<B: MessageBody>(res: &ServiceResponse<B>, request_id: String, start_time: Instant) {
    let duration = start_time.elapsed();
    let status_code = res.status().as_u16();
    
    if status_code >= 400 {
        warn!(
            request_id = %request_id,
            status = %status_code,
            duration_ms = %duration.as_millis(),
            "Request failed"
        );
    } else {
        info!(
            request_id = %request_id,
            status = %status_code,
            duration_ms = %duration.as_millis(),
            "Request completed"
        );
    }
}