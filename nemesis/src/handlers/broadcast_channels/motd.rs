// src/handlers/auth.rs
use actix_web::{HttpRequest, HttpResponse, web, get};
use std::time::SystemTime;
use tokio::time;

#[get("/get_server_time")]
pub async fn get(req: HttpRequest) -> HttpResponse {
    // Check if client accepts binary response
    let accept_header = req.headers().get("Accept");
    let accept_binary = accept_header
        .and_then(|h| h.to_str().ok())
        .map(|s| s.contains("application/x-ag-binary"))
        .unwrap_or(false);
    
    if accept_binary {
    
        // Return binary encoded response
        let response_value = serde_json::json!({
            "cursor": 0,
            "count": 1,
            "start": 0,
            "total": 1,
            "results": [
                {
                    "broadcast_slug": "featured-wbplay0515",
                    "broadcast_channel_slug": "motd",
                    "id": "test-motd",
                    "broadcast": {
                        "name": "Test Message",
                        "created_at": SystemTime::now()
                            .duration_since(SystemTime::UNIX_EPOCH)
                            .unwrap()
                            .as_millis(),
                        "updated_at": SystemTime::now()
                            .duration_since(SystemTime::UNIX_EPOCH)
                            .unwrap()
                            .as_millis(),
                        "slug": "test-motd",
                    },
                    "data": {
                        "ps4jp_product": "",
                        "file_slug": "test-motd",
                        "title": {
                            "enUS": "Test Message",
                        },
                        "bundle": "social/motd/featured-wbplay0515.bndl",
                        "ps4_product": "",
                        "text": {
                            "enUS": "Test Message Body",
                        },
                        "version": 8,
                        "action": "",
                        "file_global": false,
                        "link": "",
                    },
                    "start_at": SystemTime::now()
                            .duration_since(SystemTime::UNIX_EPOCH)
                            .unwrap()
                            .as_millis(),
                    "end_at": SystemTime::now()
                            .duration_since(SystemTime::UNIX_EPOCH)
                            .unwrap()
                            .as_millis() + 1000 * 60 * 60 * 24,
                    "content": "Test Message Body Content",
                }
            ]
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