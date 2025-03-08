// src/handlers/auth.rs
use actix_web::{HttpRequest, HttpResponse, web, get};
use serde::{Deserialize};
use std::time::SystemTime;
use tokio::time;

#[derive(Deserialize)]
struct QueryParams {
    query_start_MS: Option<String>,
}

#[get("/get_server_time")]
pub async fn get(query: web::Query<QueryParams>, req: HttpRequest) -> HttpResponse {
    // Get `query_start_MS` from GET parameters
    let query_start_ms = query.query_start_MS.clone();

    // Check if client accepts binary response
    let accept_header = req.headers().get("Accept");
    let accept_binary = accept_header
        .and_then(|h| h.to_str().ok())
        .map(|s| s.contains("application/x-ag-binary"))
        .unwrap_or(false);
    
    if accept_binary {
    

        // Return binary encoded response
        let response_value = serde_json::json!({
            "server_time": SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis(),
            "query_start_MS": query_start_ms
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
            "Error": "Unsupported media type"
        }))
    }
}