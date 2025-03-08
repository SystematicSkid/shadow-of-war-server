// src/handlers/default.rs
use actix_web::{HttpRequest, HttpResponse, http::StatusCode};
use crate::codec::encoder::encode_value;
use serde_json::json;

pub async fn not_found_handler(req: HttpRequest) -> HttpResponse {
    // Determine if the client accepts binary responses
    let accept_header = req.headers().get("Accept");
    let accept_binary = accept_header
        .and_then(|h| h.to_str().ok())
        .map(|s| s.contains("application/x-ag-binary"))
        .unwrap_or(false);
    
    // Create the error response payload
    /*
  "code": 400,
  "msg": "Invalid Steam Ticket",
  "hydra_error": 16,
  "relying_party_error": 0,
    */
    let error_response = json!({
        "code": 404,
        "msg": format!("Route not found: {}", req.path()),
        "hydra_error": 16,
        "relying_party_error": 0,
        "body": {}
    });
    
    if accept_binary {
        // Return binary encoded response
        match encode_value(&error_response) {
            Ok(binary_data) => {
                HttpResponse::NotFound()
                    .content_type("application/x-ag-binary")
                    .body(binary_data)
            },
            Err(_) => HttpResponse::InternalServerError().finish()
        }
    } else {
        // Return JSON response
        HttpResponse::NotFound()
            .json(error_response)
    }
}