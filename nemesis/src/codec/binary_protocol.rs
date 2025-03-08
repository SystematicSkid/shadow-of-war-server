use std::collections::HashMap;
use std::io::{Cursor, Read, Write};
use bytes::{Bytes, BytesMut, Buf, BufMut};
use chrono::{DateTime, Utc, TimeZone};
use serde_json::{Value, Map, json};
use anyhow::{Result, anyhow, Context};
use std::convert::TryFrom;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use tracing::{debug, trace, warn};

// Type identifiers as constants
pub const TYPE_PASS: u8 = 0x01;
pub const TYPE_BOOLEAN_FALSE: u8 = 0x02;
pub const TYPE_BOOLEAN_TRUE: u8 = 0x03;
pub const TYPE_U1: u8 = 0x10;  // 1 byte unsigned integer
pub const TYPE_U2: u8 = 0x12;  // 2 byte unsigned integer
pub const TYPE_U4: u8 = 0x14;  // 4 byte unsigned integer
pub const TYPE_S1: u8 = 0x11;  // 1 byte signed integer
pub const TYPE_S2: u8 = 0x13;  // 2 byte signed integer
pub const TYPE_S4: u8 = 0x15;  // 4 byte signed integer
pub const TYPE_S8: u8 = 0x16;  // 8 byte signed integer
pub const TYPE_U8: u8 = 0x17;  // 8 byte unsigned integer
pub const TYPE_FLOAT: u8 = 0x20;  // 4 byte float
pub const TYPE_DOUBLE: u8 = 0x21;  // 8 byte double
pub const TYPE_STRING: u8 = 0x30;  // String (1 byte length + chars)
pub const TYPE_LONG_STRING: u8 = 0x31;  // String (2 byte length + chars)
pub const TYPE_BINARY_DATA_U1: u8 = 0x33;  // Binary data (1 byte length + data)
pub const TYPE_BINARY_DATA_U2: u8 = 0x34;  // Binary data (2 byte length + data)
pub const TYPE_BINARY_DATA_U4: u8 = 0x35;  // Binary data (4 byte length + data)
pub const TYPE_DATETIME: u8 = 0x40;  // DateTime
pub const TYPE_NULL: u8 = 0x41;  // Null value
pub const TYPE_ARRAY: u8 = 0x50;  // Array (1 byte size + elements)
pub const TYPE_LONG_ARRAY: u8 = 0x51;  // Array (2 byte size + elements)
pub const TYPE_MAP: u8 = 0x60;  // Map (1 byte size + key-value pairs)
pub const TYPE_LONG_MAP: u8 = 0x61;  // Map (2 byte size + key-value pairs)
pub const TYPE_COMPRESSED: u8 = 0x67;  // Compressed data

pub struct BinaryProtocol {
    buffer: Cursor<Vec<u8>>,
}

impl BinaryProtocol {
    pub fn new() -> Self {
        BinaryProtocol {
            buffer: Cursor::new(Vec::new()),
        }
    }
    
    pub fn set_buffer(&mut self, data: Vec<u8>) {
        self.buffer = Cursor::new(data);
    }
    
    pub fn get_buffer(&self) -> Vec<u8> {
        self.buffer.get_ref().clone()
    }
    
    pub fn reset(&mut self) {
        self.buffer = Cursor::new(Vec::new());
    }
    
    // Read methods
    pub fn read_type(&mut self) -> Result<u8> {
        let mut byte = [0u8; 1];
        self.buffer.read_exact(&mut byte)?;
        Ok(byte[0])
    }
    
    pub fn read_u1(&mut self) -> Result<u8> {
        let mut byte = [0u8; 1];
        self.buffer.read_exact(&mut byte)?;
        Ok(byte[0])
    }
    
    pub fn read_u2(&mut self) -> Result<u16> {
        let mut bytes = [0u8; 2];
        self.buffer.read_exact(&mut bytes)?;
        Ok(u16::from_be_bytes(bytes))
    }
    
    pub fn read_u4(&mut self) -> Result<u32> {
        let mut bytes = [0u8; 4];
        self.buffer.read_exact(&mut bytes)?;
        Ok(u32::from_be_bytes(bytes))
    }
    
    pub fn read_s1(&mut self) -> Result<i8> {
        let mut byte = [0u8; 1];
        self.buffer.read_exact(&mut byte)?;
        Ok(i8::from_be_bytes(byte))
    }
    
    pub fn read_s2(&mut self) -> Result<i16> {
        let mut bytes = [0u8; 2];
        self.buffer.read_exact(&mut bytes)?;
        Ok(i16::from_be_bytes(bytes))
    }
    
    pub fn read_s4(&mut self) -> Result<i32> {
        let mut bytes = [0u8; 4];
        self.buffer.read_exact(&mut bytes)?;
        Ok(i32::from_be_bytes(bytes))
    }
    
    pub fn read_s8(&mut self) -> Result<i64> {
        let mut bytes = [0u8; 8];
        self.buffer.read_exact(&mut bytes)?;
        Ok(i64::from_be_bytes(bytes))
    }
    
    pub fn read_u8(&mut self) -> Result<u64> {
        let mut bytes = [0u8; 8];
        self.buffer.read_exact(&mut bytes)?;
        Ok(u64::from_be_bytes(bytes))
    }
    
    pub fn read_float(&mut self) -> Result<f32> {
        let mut bytes = [0u8; 4];
        self.buffer.read_exact(&mut bytes)?;
        Ok(f32::from_be_bytes(bytes))
    }
    
    pub fn read_double(&mut self) -> Result<f64> {
        let mut bytes = [0u8; 8];
        self.buffer.read_exact(&mut bytes)?;
        Ok(f64::from_be_bytes(bytes))
    }
    
    pub fn read_binary_data_u1(&mut self) -> Result<Vec<u8>> {
        let length = self.read_u1()?;
        let mut data = vec![0u8; length as usize];
        self.buffer.read_exact(&mut data)?;
        Ok(data)
    }
    
    pub fn read_binary_data_u2(&mut self) -> Result<Vec<u8>> {
        let length = self.read_u2()?;
        let mut data = vec![0u8; length as usize];
        self.buffer.read_exact(&mut data)?;
        Ok(data)
    }
    
    pub fn read_binary_data_u4(&mut self) -> Result<Vec<u8>> {
        let length = self.read_u4()?;
        let mut data = vec![0u8; length as usize];
        self.buffer.read_exact(&mut data)?;
        Ok(data)
    }
    
    pub fn read_datetime(&mut self) -> Result<DateTime<Utc>> {
        let timestamp = self.read_u4()?;
        Ok(Utc.timestamp_opt(timestamp as i64, 0).unwrap())
    }
    
    pub fn read_string(&mut self) -> Result<String> {
        let length = self.read_u1()?;
        let mut data = vec![0u8; length as usize];
        self.buffer.read_exact(&mut data)?;
        String::from_utf8(data).map_err(|e| anyhow!("Invalid UTF-8: {}", e))
    }
    
    pub fn read_long_string(&mut self) -> Result<String> {
        let length = self.read_u2()?;
        let mut data = vec![0u8; length as usize];
        self.buffer.read_exact(&mut data)?;
        String::from_utf8(data).map_err(|e| anyhow!("Invalid UTF-8: {}", e))
    }
    
    pub fn read_array(&mut self) -> Result<Vec<Value>> {
        let size = self.read_u1()?;
        let mut result = Vec::with_capacity(size as usize);
        for _ in 0..size {
            result.push(self.read_value()?);
        }
        Ok(result)
    }
    
    pub fn read_long_array(&mut self) -> Result<Vec<Value>> {
        let size = self.read_u2()?;
        let mut result = Vec::with_capacity(size as usize);
        for _ in 0..size {
            result.push(self.read_value()?);
        }
        Ok(result)
    }
    
    pub fn read_map(&mut self) -> Result<Map<String, Value>> {
        let size = self.read_u1()?;
        let mut result = Map::new();
        for _ in 0..size {
            let key = self.read_value()?;
            let value = self.read_value()?;
            
            let key_str = match key {
                Value::String(s) => s,
                Value::Number(n) => n.to_string(),
                _ => {
                    warn!("Non-string/number key in map, converting to string: {:?}", key);
                    key.to_string()
                }
            };
            
            result.insert(key_str, value);
        }
        Ok(result)
    }
    
    pub fn read_long_map(&mut self) -> Result<Map<String, Value>> {
        let size = self.read_u2()?;
        let mut result = Map::new();
        for _ in 0..size {
            let key = self.read_value()?;
            let value = self.read_value()?;
            
            let key_str = match key {
                Value::String(s) => s,
                Value::Number(n) => n.to_string(),
                _ => {
                    warn!("Non-string/number key in map, converting to string: {:?}", key);
                    key.to_string()
                }
            };
            
            result.insert(key_str, value);
        }
        Ok(result)
    }
    
    pub fn read_compressed(&mut self) -> Result<Value> {
        let _unk_byte = self.read_u1()?;
        let compressed_data = match self.read_value()? {
            Value::Array(arr) => {
                let bytes: Result<Vec<u8>> = arr.into_iter()
                    .map(|v| match v {
                        Value::Number(n) => n.as_u64()
                            .map(|n| u8::try_from(n).map_err(|_| anyhow!("Invalid byte value")))
                            .ok_or_else(|| anyhow!("Invalid numeric value"))?,
                        _ => Err(anyhow!("Expected number in byte array"))
                    })
                    .collect();
                bytes?
            },
            Value::String(s) => s.into_bytes(),
            _ => return Err(anyhow!("Unexpected compressed data format")),
        };
        
        let mut decoder = ZlibDecoder::new(&compressed_data[..]);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)?;
        
        let mut new_protocol = BinaryProtocol::new();
        new_protocol.set_buffer(decompressed);
        new_protocol.read_value()
    }
    
    pub fn read_value(&mut self) -> Result<Value> {
        let type_id = self.read_type()?;
        match type_id {
            TYPE_PASS => Ok(Value::Null),
            TYPE_BOOLEAN_FALSE => Ok(Value::Bool(false)),
            TYPE_BOOLEAN_TRUE => Ok(Value::Bool(true)),
            TYPE_U1 => {
                let value = self.read_u1()?;
                Ok(Value::Number(value.into()))
            },
            TYPE_U2 => {
                let value = self.read_u2()?;
                Ok(Value::Number(value.into()))
            },
            TYPE_U4 => {
                let value = self.read_u4()?;
                Ok(Value::Number(value.into()))
            },
            TYPE_S1 => {
                let value = self.read_s1()?;
                Ok(Value::Number(value.into()))
            },
            TYPE_S2 => {
                let value = self.read_s2()?;
                Ok(Value::Number(value.into()))
            },
            TYPE_S4 => {
                let value = self.read_s4()?;
                Ok(Value::Number(value.into()))
            },
            TYPE_S8 => {
                let value = self.read_s8()?;
                if let Some(n) = serde_json::Number::from_f64(value as f64) {
                    Ok(Value::Number(n))
                } else {
                    Ok(Value::String(value.to_string()))
                }
            },
            TYPE_U8 => {
                let value = self.read_u8()?;
                if let Some(n) = serde_json::Number::from_f64(value as f64) {
                    Ok(Value::Number(n))
                } else {
                    Ok(Value::String(value.to_string()))
                }
            },
            TYPE_FLOAT => {
                let value = self.read_float()?;
                if let Some(n) = serde_json::Number::from_f64(value as f64) {
                    Ok(Value::Number(n))
                } else {
                    Ok(Value::String(value.to_string()))
                }
            },
            TYPE_DOUBLE => {
                let value = self.read_double()?;
                if let Some(n) = serde_json::Number::from_f64(value) {
                    Ok(Value::Number(n))
                } else {
                    Ok(Value::String(value.to_string()))
                }
            },
            TYPE_STRING => {
                let value = self.read_string()?;
                Ok(Value::String(value))
            },
            TYPE_LONG_STRING => {
                let value = self.read_long_string()?;
                Ok(Value::String(value))
            },
            TYPE_BINARY_DATA_U1 => {
                let data = self.read_binary_data_u1()?;
                Ok(Value::Array(data.into_iter().map(|b| Value::Number(b.into())).collect()))
            },
            TYPE_BINARY_DATA_U2 => {
                let data = self.read_binary_data_u2()?;
                Ok(Value::Array(data.into_iter().map(|b| Value::Number(b.into())).collect()))
            },
            TYPE_BINARY_DATA_U4 => {
                let data = self.read_binary_data_u4()?;
                Ok(Value::Array(data.into_iter().map(|b| Value::Number(b.into())).collect()))
            },
            TYPE_DATETIME => {
                let dt = self.read_datetime()?;
                Ok(Value::String(dt.to_rfc3339()))
            },
            TYPE_NULL => Ok(Value::Null),
            TYPE_ARRAY => {
                let array = self.read_array()?;
                Ok(Value::Array(array))
            },
            TYPE_LONG_ARRAY => {
                let array = self.read_long_array()?;
                Ok(Value::Array(array))
            },
            TYPE_MAP => {
                let map = self.read_map()?;
                Ok(Value::Object(map))
            },
            TYPE_LONG_MAP => {
                let map = self.read_long_map()?;
                Ok(Value::Object(map))
            },
            TYPE_COMPRESSED => {
                self.read_compressed()
            },
            _ => Err(anyhow!("Unknown type identifier: 0x{:02x}", type_id)),
        }
    }
    
    // Write methods
    pub fn write_type(&mut self, type_id: u8) -> Result<()> {
        self.buffer.write_all(&[type_id])?;
        Ok(())
    }
    
    pub fn write_u1(&mut self, value: u8) -> Result<()> {
        self.buffer.write_all(&[value])?;
        Ok(())
    }
    
    pub fn write_u2(&mut self, value: u16) -> Result<()> {
        self.buffer.write_all(&value.to_be_bytes())?;
        Ok(())
    }
    
    pub fn write_u4(&mut self, value: u32) -> Result<()> {
        self.buffer.write_all(&value.to_be_bytes())?;
        Ok(())
    }
    
    pub fn write_s1(&mut self, value: i8) -> Result<()> {
        self.buffer.write_all(&value.to_be_bytes())?;
        Ok(())
    }
    
    pub fn write_s2(&mut self, value: i16) -> Result<()> {
        self.buffer.write_all(&value.to_be_bytes())?;
        Ok(())
    }
    
    pub fn write_s4(&mut self, value: i32) -> Result<()> {
        self.buffer.write_all(&value.to_be_bytes())?;
        Ok(())
    }
    
    pub fn write_s8(&mut self, value: i64) -> Result<()> {
        self.buffer.write_all(&value.to_be_bytes())?;
        Ok(())
    }
    
    pub fn write_u8(&mut self, value: u64) -> Result<()> {
        self.buffer.write_all(&value.to_be_bytes())?;
        Ok(())
    }
    
    pub fn write_float(&mut self, value: f32) -> Result<()> {
        self.buffer.write_all(&value.to_be_bytes())?;
        Ok(())
    }
    
    pub fn write_double(&mut self, value: f64) -> Result<()> {
        self.buffer.write_all(&value.to_be_bytes())?;
        Ok(())
    }
    
    pub fn write_string(&mut self, value: &str) -> Result<()> {
        let bytes = value.as_bytes();
        if bytes.len() > 255 {
            return Err(anyhow!("String too long for write_string (max 255 bytes)"));
        }
        self.write_u1(bytes.len() as u8)?;
        self.buffer.write_all(bytes)?;
        Ok(())
    }
    
    pub fn write_long_string(&mut self, value: &str) -> Result<()> {
        let bytes = value.as_bytes();
        if bytes.len() > 65535 {
            return Err(anyhow!("String too long for write_long_string (max 65535 bytes)"));
        }
        self.write_u2(bytes.len() as u16)?;
        self.buffer.write_all(bytes)?;
        Ok(())
    }
    
    pub fn write_array(&mut self, values: &[Value]) -> Result<()> {
        if values.len() > 255 {
            return Err(anyhow!("Array too large for write_array (max 255 elements)"));
        }
        self.write_u1(values.len() as u8)?;
        for value in values {
            self.write_value(value)?;
        }
        Ok(())
    }
    
    pub fn write_long_array(&mut self, values: &[Value]) -> Result<()> {
        if values.len() > 65535 {
            return Err(anyhow!("Array too large for write_long_array (max 65535 elements)"));
        }
        self.write_u2(values.len() as u16)?;
        for value in values {
            self.write_value(value)?;
        }
        Ok(())
    }
    
    pub fn write_map(&mut self, map: &Map<String, Value>) -> Result<()> {
        if map.len() > 255 {
            return Err(anyhow!("Map too large for write_map (max 255 entries)"));
        }
        self.write_u1(map.len() as u8)?;
        for (key, value) in map {
            self.write_value(&Value::String(key.clone()))?;
            self.write_value(value)?;
        }
        Ok(())
    }
    
    pub fn write_long_map(&mut self, map: &Map<String, Value>) -> Result<()> {
        if map.len() > 65535 {
            return Err(anyhow!("Map too large for write_long_map (max 65535 entries)"));
        }
        self.write_u2(map.len() as u16)?;
        for (key, value) in map {
            self.write_value(&Value::String(key.clone()))?;
            self.write_value(value)?;
        }
        Ok(())
    }
    
    pub fn write_compressed(&mut self, value: &Value) -> Result<()> {
        let mut inner_protocol = BinaryProtocol::new();
        inner_protocol.write_value(value)?;
        let encoded = inner_protocol.get_buffer();
        
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&encoded)?;
        let compressed = encoder.finish()?;
        
        self.write_type(TYPE_COMPRESSED)?;
        self.write_u1(1)?; // Unknown byte, no idea right now
        
        if compressed.len() <= 255 {
            self.write_type(TYPE_BINARY_DATA_U1)?;
            self.write_u1(compressed.len() as u8)?;
        } else if compressed.len() <= 65535 {
            self.write_type(TYPE_BINARY_DATA_U2)?;
            self.write_u2(compressed.len() as u16)?;
        } else {
            self.write_type(TYPE_BINARY_DATA_U4)?;
            self.write_u4(compressed.len() as u32)?;
        }
        self.buffer.write_all(&compressed)?;
        
        Ok(())
    }
    
    pub fn write_value(&mut self, value: &Value) -> Result<()> {
        match value {
            Value::Null => {
                self.write_type(TYPE_NULL)?;
            },
            Value::Bool(b) => {
                if *b {
                    self.write_type(TYPE_BOOLEAN_TRUE)?;
                } else {
                    self.write_type(TYPE_BOOLEAN_FALSE)?;
                }
            },
            Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    if i >= -128 && i <= 127 {
                        self.write_type(TYPE_S1)?;
                        self.write_s1(i as i8)?;
                    } else if i >= -32768 && i <= 32767 {
                        self.write_type(TYPE_S2)?;
                        self.write_s2(i as i16)?;
                    } else if i >= -2147483648 && i <= 2147483647 {
                        self.write_type(TYPE_S4)?;
                        self.write_s4(i as i32)?;
                    } else {
                        self.write_type(TYPE_S8)?;
                        self.write_s8(i)?;
                    }
                } else if let Some(u) = n.as_u64() {
                    if u <= 255 {
                        self.write_type(TYPE_U1)?;
                        self.write_u1(u as u8)?;
                    } else if u <= 65535 {
                        self.write_type(TYPE_U2)?;
                        self.write_u2(u as u16)?;
                    } else if u <= 4294967295 {
                        self.write_type(TYPE_U4)?;
                        self.write_u4(u as u32)?;
                    } else {
                        self.write_type(TYPE_U8)?;
                        self.write_u8(u)?;
                    }
                } else if let Some(f) = n.as_f64() {
                    self.write_type(TYPE_DOUBLE)?;
                    self.write_double(f)?;
                } else {
                    return Err(anyhow!("Unsupported number format"));
                }
            },
            Value::String(s) => {
                let bytes = s.as_bytes();
                if bytes.len() <= 255 {
                    self.write_type(TYPE_STRING)?;
                    self.write_string(s)?;
                } else {
                    self.write_type(TYPE_LONG_STRING)?;
                    self.write_long_string(s)?;
                }
            },
            Value::Array(arr) => {
                if arr.len() <= 255 {
                    self.write_type(TYPE_ARRAY)?;
                    self.write_array(arr)?;
                } else {
                    self.write_type(TYPE_LONG_ARRAY)?;
                    self.write_long_array(arr)?;
                }
            },
            Value::Object(map) => {
                if map.len() <= 255 {
                    self.write_type(TYPE_MAP)?;
                    self.write_map(map)?;
                } else {
                    self.write_type(TYPE_LONG_MAP)?;
                    self.write_long_map(map)?;
                }
            },
        }
        
        Ok(())
    }
    
    pub fn encode_json(&mut self, data: &Value) -> Result<Vec<u8>> {
        self.reset();
        self.write_value(data)?;
        Ok(self.get_buffer())
    }
    
    pub fn decode_to_json(&mut self) -> Result<Value> {
        self.read_value()
    }
}

impl Default for BinaryProtocol {
    fn default() -> Self {
        Self::new()
    }
}