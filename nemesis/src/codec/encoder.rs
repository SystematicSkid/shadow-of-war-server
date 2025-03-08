use bytes::Bytes;
use serde::Serialize;
use serde_json::Value;
use anyhow::{Result, anyhow};

use crate::codec::binary_protocol::BinaryProtocol;

/// Encodes a value into the game's custom binary format
pub fn encode<T>(data: &T) -> Result<Bytes>
where
    T: Serialize,
{
    // Convert to serde_json::Value for encoding
    let value = serde_json::to_value(data)
        .map_err(|e| anyhow!("Failed to serialize data to JSON: {}", e))?;
    
    // Use the binary protocol to encode
    let mut protocol = BinaryProtocol::new();
    let encoded = protocol.encode_json(&value)?;
    
    Ok(Bytes::from(encoded))
}

/// Encodes a JSON Value into the game's custom binary format directly
pub fn encode_value(value: &Value) -> Result<Bytes> {
    let mut protocol = BinaryProtocol::new();
    let encoded = protocol.encode_json(value)?;
    
    Ok(Bytes::from(encoded))
}