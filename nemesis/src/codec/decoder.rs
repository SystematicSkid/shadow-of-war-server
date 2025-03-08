use bytes::Bytes;
use serde::de::DeserializeOwned;
use serde_json::Value;
use anyhow::{Result, anyhow, Context};
use tracing::{debug, error};

use crate::codec::binary_protocol::BinaryProtocol;

pub fn decode<T>(bytes: Bytes) -> Result<T>
where
    T: DeserializeOwned,
{
    let value = decode_to_value(bytes)?;
    
    serde_json::from_value(value)
        .with_context(|| "Failed to deserialize decoded data to target type".to_string())
}

/// Decodes binary data from the game's custom format into a JSON Value
pub fn decode_to_value(bytes: Bytes) -> Result<Value> {
    let mut protocol = BinaryProtocol::new();
    protocol.set_buffer(bytes.to_vec());
    
    protocol.decode_to_json()
        .with_context(|| "Failed to decode binary data".to_string())
}