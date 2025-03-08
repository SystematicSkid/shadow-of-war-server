use std::future::Future;
use std::pin::Pin;

use actix_web::{web, FromRequest, HttpRequest, dev::Payload};
use bytes::Bytes;
use serde::de::DeserializeOwned;
use serde_json::Value;
use anyhow::Context;
use tracing::{debug, error};

use crate::codec::{decode_to_value, decode};

/// Extractor for binary format requests
/// This extractor will automatically decode binary requests to the specified type
pub struct BinaryRequest<T>(pub T);

impl<T> BinaryRequest<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> FromRequest for BinaryRequest<T>
where
    T: DeserializeOwned + 'static,
{
    type Error = actix_web::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        let req2 = req.clone();
        let mut payload = payload.take();

        Box::pin(async move {
            let bytes = web::Bytes::from_request(&req2, &mut payload).await?;
            
            // Check if we have the binary content type
            let is_binary = req2
                .headers()
                .get("Content-Type")
                .and_then(|h| h.to_str().ok())
                .map(|s| s.contains("application/x-ag-binary"))
                .unwrap_or(false);
            
            if is_binary {
                debug!("Decoding binary request");
                let data = decode::<T>(bytes).map_err(|e| {
                    error!("Failed to decode binary data: {}", e);
                    actix_web::error::ErrorBadRequest(format!("Failed to decode binary data: {}", e))
                })?;
                
                Ok(BinaryRequest(data))
            } else {
                // If it's not binary, assume it's JSON and from some other source, we log it anyhow
                let data: T = serde_json::from_slice(&bytes).map_err(|e| {
                    error!("Failed to deserialize JSON: {}", e);
                    actix_web::error::ErrorBadRequest(format!("Failed to deserialize JSON: {}", e))
                })?;
                
                Ok(BinaryRequest(data))
            }
        })
    }
}

/// Value extractor for binary format
/// Similar to BinaryRequest, but always returns serde_json::Value
pub struct BinaryValue(pub Value);

impl BinaryValue {
    pub fn into_inner(self) -> Value {
        self.0
    }
}

impl FromRequest for BinaryValue {
    type Error = actix_web::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        let req2 = req.clone();
        let mut payload = payload.take();

        Box::pin(async move {
            let bytes = web::Bytes::from_request(&req2, &mut payload).await?;
            
            // Check if we have the binary content type
            let is_binary = req2
                .headers()
                .get("Content-Type")
                .and_then(|h| h.to_str().ok())
                .map(|s| s.contains("application/x-ag-binary"))
                .unwrap_or(false);
            
            if is_binary {
                debug!("Decoding binary request to Value");
                let value = decode_to_value(bytes).map_err(|e| {
                    error!("Failed to decode binary data to Value: {}", e);
                    actix_web::error::ErrorBadRequest(format!("Failed to decode binary data: {}", e))
                })?;
                
                Ok(BinaryValue(value))
            } else {
                // If it's not binary, assume it's JSON and from some other source, we log it anyhow
                let value: Value = serde_json::from_slice(&bytes).map_err(|e| {
                    error!("Failed to deserialize JSON to Value: {}", e);
                    actix_web::error::ErrorBadRequest(format!("Failed to deserialize JSON: {}", e))
                })?;
                
                Ok(BinaryValue(value))
            }
        })
    }
}