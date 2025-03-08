// src/handlers/auth.rs
use actix_web::{HttpRequest, HttpResponse};
use crate::codec::extractor::BinaryValue;
use serde_json::Value;

pub async fn authenticate(body: BinaryValue, req: HttpRequest) -> HttpResponse {
    // Extract steam token from body.0
    let steam_token = body.0.get("steam").and_then(Value::as_str);
    
    // Check if client accepts binary response
    let accept_header = req.headers().get("Accept");
    let accept_binary = accept_header
        .and_then(|h| h.to_str().ok())
        .map(|s| s.contains("application/x-ag-binary"))
        .unwrap_or(false);
    
    if accept_binary {

        // Return binary encoded response, this is hard-coded for now as a test
        let response_value = serde_json::json!({
            "created_at": "2019-12-27 21:21:47",
            "updated_at": "2019-12-27 21:21:47",
            "token": "11111111111111111111111",
            "type": "steam",
            "account_id": "5e06ca4bd417094251b9cff1",
            "auth_id": "76561193757519789",
            "expires_at": null,
            "fingerprint": null,
            "id": "5e06ca4bd417094251b9cff2"
        });
        
        // Encode to binary
        match crate::codec::encoder::encode_value(&response_value) {
            Ok(binary_data) => {
                HttpResponse::Ok()
                    .content_type("application/x-ag-binary")
                    .body(binary_data)
            },
            Err(_) => HttpResponse::InternalServerError().finish()
        }
    } else {
        // Return JSON response
        HttpResponse::Ok().json(serde_json::json!({
            "info": "This endpoint is not supported in non-binary mode"
        }))
    }
}