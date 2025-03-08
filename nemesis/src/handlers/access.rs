use actix_web::{HttpRequest, HttpResponse};
use crate::codec::extractor::BinaryValue;
use serde_json::Value;

pub async fn get_token(body: BinaryValue, req: HttpRequest) -> HttpResponse {
    // Ensure there is an 'auth_token' in the body
    let auth_token = body.0.get("auth_token");
    if auth_token.is_none() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Missing auth_token"
        }));
    }
    
    // Check if client accepts binary response
    let accept_header = req.headers().get("Accept");
    let accept_binary = accept_header
        .and_then(|h| h.to_str().ok())
        .map(|s| s.contains("application/x-ag-binary"))
        .unwrap_or(false);
    
    if accept_binary {
    

        // Return binary encoded response
        let response_value = serde_json::json!({
            "token": "sNcqvKhNlrONTZ3Yd3RgsvyrnLatHzmeMW9Pkxe3iWHyBV5x7fP3Hifn4mc+3Y7x6lpICGzPdILW/keD0aDf/nstQDWiT5m6QhRBYHkA/N4XYDs8/FhU0f8H53zkKy0H"
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
