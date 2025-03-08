import struct
import json
import requests
from datetime import datetime
from io import BytesIO
from typing import Dict, List, Any, Union, Tuple
# Zlib inflate
import zlib

class BinaryProtocol:
    """Handler for application/x-ag-binary protocol encoding/decoding"""
    
    # Type identifiers
    TYPE_PASS = 0x01
    TYPE_BOOLEAN_FALSE = 0x02
    TYPE_BOOLEAN_TRUE = 0x03
    TYPE_U1 = 0x10  # 1 byte unsigned integer
    TYPE_U2 = 0x12  # 2 byte unsigned integer
    TYPE_U4 = 0x14  # 4 byte unsigned integer
    TYPE_S1 = 0x11  # 1 byte signed integer
    TYPE_S2 = 0x13  # 2 byte signed integer
    TYPE_S4 = 0x15  # 4 byte signed integer
    TYPE_S8 = 0x16  # 8 byte signed integer
    TYPE_U8 = 0x17  # 8 byte unsigned integer
    TYPE_FLOAT = 0x20  # 4 byte float
    TYPE_DOUBLE = 0x21  # 8 byte double
    TYPE_STRING = 0x30  # String (1 byte length + chars)
    TYPE_LONG_STRING = 0x31  # String (2 byte length + chars)
    TYPE_BINARY_DATA_U1 = 0x33  # Binary data (1 byte length + data)
    TYPE_BINARY_DATA_U2 = 0x34  # Binary data (2 byte length + data)
    TYPE_BINARY_DATA_U4 = 0x35  # Binary data (4 byte length + data)
    TYPE_DATETIME = 0x40  # DateTime
    TYPE_NULL = 0x41  # Null value
    TYPE_ARRAY = 0x50  # Array (1 byte size + elements)
    TYPE_LONG_ARRAY = 0x51  # Array (2 byte size + elements)
    TYPE_MAP = 0x60  # Map (1 byte size + key-value pairs)
    TYPE_LONG_MAP = 0x61  # Map (2 byte size + key-value pairs)
    TYPE_COMPRESSED =0x67  # Compressed data (variable size type, variable size content)
    
    def __init__(self):
        self.buffer = BytesIO()
        self.position = 0
    
    def set_buffer(self, data: bytes) -> None:
        """Set the buffer with the provided binary data"""
        self.buffer = BytesIO(data)
        self.position = 0
    
    def get_buffer(self) -> bytes:
        """Get the current buffer as bytes"""
        return self.buffer.getvalue()
    
    def reset(self) -> None:
        """Reset the buffer"""
        self.buffer = BytesIO()
        self.position = 0
    
    # Read methods
    def read_type(self) -> int:
        """Read the type identifier byte"""
        return self._read_bytes(1)[0]
    
    def read_pass(self) -> None:
        """Read a pass (0 bytes)"""
        pass
    
    def read_u1(self) -> int:
        """Read an unsigned byte (1 byte)"""
        return self._read_bytes(1)[0]
    
    def read_u2(self) -> int:
        """Read an unsigned short (2 bytes)"""
        return struct.unpack('>H', self._read_bytes(2))[0]
    
    def read_u4(self) -> int:
        """Read an unsigned int (4 bytes)"""
        return struct.unpack('>I', self._read_bytes(4))[0]
    
    def read_s1(self) -> int:
        """Read a signed byte (1 byte)"""
        return struct.unpack('>b', self._read_bytes(1))[0]
    
    def read_s2(self) -> int:
        """Read a signed short (2 bytes)"""
        return struct.unpack('>h', self._read_bytes(2))[0]
    
    def read_s4(self) -> int:
        """Read a signed int (4 bytes)"""
        return struct.unpack('>i', self._read_bytes(4))[0]
    
    def read_s8(self) -> int:
        """Read a signed long (8 bytes)"""
        return struct.unpack('>q', self._read_bytes(8))[0]
    
    def read_u8(self) -> int:
        """Read an unsigned long (8 bytes)"""
        return struct.unpack('>Q', self._read_bytes(8))[0]
    
    def read_float(self) -> float:
        """Read a float (4 bytes)"""
        return struct.unpack('>f', self._read_bytes(4))[0]
    
    def read_double(self) -> float:
        """Read a double (8 bytes)"""
        return struct.unpack('>d', self._read_bytes(8))[0]
    
    def read_boolean(self) -> bool:
        """Read a boolean (1 byte)"""
        return bool(self._read_bytes(1)[0])
    
    def read_binary_data_u1(self) -> bytes:
        """Read a binary data (1 byte length + data)"""
        length = self.read_u1()
        return self._read_bytes(length)
    
    def read_binary_data_u2(self) -> bytes: 
        """Read a binary data (2 byte length + data)"""
        length = self.read_u2()
        return self._read_bytes(length)
    
    def read_binary_data_u4(self) -> bytes:
        """Read a binary data (4 byte length + data)"""
        length = self.read_u4()
        return self._read_bytes(length)
    
    def read_datetime(self) -> str:
        """Read a datetime (4 bytes)"""
        date_time = self.read_u4()
        return datetime.fromtimestamp(date_time).strftime('%Y-%m-%d %H:%M:%S')
    
    def read_string(self) -> str:
        """Read a string with 1-byte length prefix"""
        length = self.read_u1()
        return self._read_bytes(length).decode('utf-8')
    
    def read_long_string(self) -> str:
        """Read a string with 2-byte length prefix"""
        length = self.read_u2()
        return self._read_bytes(length).decode('utf-8')
    
    def read_array(self) -> List[Any]:
        """Read an array with 1-byte size prefix"""
        size = self.read_u1()
        return [self.read_value() for _ in range(size)]
    
    def read_long_array(self) -> List[Any]:
        """Read an array with 2-byte size prefix"""
        size = self.read_u2()
        return [self.read_value() for _ in range(size)]
    
    def read_map(self) -> Dict[str, Any]:
        """Read a map with 1-byte size prefix"""
        size = self.read_u1()
        result = {}
        for _ in range(size):
            key = self.read_value()
            value = self.read_value()
            if isinstance(key, (str, int)):
                result[str(key)] = value
        return result
    
    def read_long_map(self) -> Dict[str, Any]:
        """Read a map with 2-byte size prefix"""
        size = self.read_u2()
        result = {}
        for _ in range(size):
            key = self.read_value()
            value = self.read_value()
            if isinstance(key, (str, int)):
                result[str(key)] = value
        return result
    
    def read_compressed(self) -> bytes:
        """Read a compressed (variable size type, variable size content)"""
        unk_byte = self.read_u1()
        compressed_data = self.read_value()
        decompressed_data = zlib.decompress(compressed_data)
        # Dump decompressed data to a file
        with open('decompressed_data.bin', 'wb') as f:
            f.write(decompressed_data)
        # decompressed data is more protocol data, so we need to decode it
        new_protocol = BinaryProtocol()
        new_protocol.set_buffer(decompressed_data)
        return new_protocol.read_value()
    
    def read_value(self) -> Any:
        """Read a value based on its type identifier"""
        type_id = self.read_type()
        if type_id == self.TYPE_PASS:
            return self.read_pass()
        elif type_id == self.TYPE_BOOLEAN_FALSE:
            return False
        elif type_id == self.TYPE_BOOLEAN_TRUE:
            return True
        elif type_id == self.TYPE_U1:
            return self.read_u1()
        elif type_id == self.TYPE_U2:
            return self.read_u2()
        elif type_id == self.TYPE_U4:
            return self.read_u4()
        elif type_id == self.TYPE_S1:
            return self.read_s1()
        elif type_id == self.TYPE_S2:
            return self.read_s2()
        elif type_id == self.TYPE_S4:
            return self.read_s4()
        elif type_id == self.TYPE_S8:
            return self.read_s8()
        elif type_id == self.TYPE_U8:
            return self.read_u8()
        elif type_id == self.TYPE_FLOAT:
            return self.read_float()
        elif type_id == self.TYPE_DOUBLE:
            return self.read_double()
        elif type_id == self.TYPE_STRING:
            return self.read_string()
        elif type_id == self.TYPE_LONG_STRING:
            return self.read_long_string()
        #elif type_id == self.TYPE_BOOLEAN:
        #    return self.read_boolean()
        elif type_id == self.TYPE_BINARY_DATA_U1:
            return self.read_binary_data_u1()
        elif type_id == self.TYPE_BINARY_DATA_U2:
            return self.read_binary_data_u2()
        elif type_id == self.TYPE_BINARY_DATA_U4:
            return self.read_binary_data_u4()
        elif type_id == self.TYPE_DATETIME:
            return self.read_datetime()
        elif type_id == self.TYPE_NULL:
            return None
        elif type_id == self.TYPE_ARRAY:
            return self.read_array()
        elif type_id == self.TYPE_LONG_ARRAY:
            return self.read_long_array()
        elif type_id == self.TYPE_MAP:
            return self.read_map()
        elif type_id == self.TYPE_LONG_MAP:
            return self.read_long_map()
        elif type_id == self.TYPE_COMPRESSED:
            return self.read_compressed()
        else:
            raise ValueError(f"Unknown type identifier: 0x{type_id:02x}")
    
    # Write methods
    def write_type(self, type_id: int) -> None:
        """Write a type identifier byte"""
        self._write_bytes(bytes([type_id]))
    
    def write_u1(self, value: int) -> None:
        """Write an unsigned byte (1 byte)"""
        self._write_bytes(bytes([value & 0xFF]))
    
    def write_u2(self, value: int) -> None:
        """Write an unsigned short (2 bytes)"""
        self._write_bytes(struct.pack('>H', value & 0xFFFF))
    
    def write_u4(self, value: int) -> None:
        """Write an unsigned int (4 bytes)"""
        self._write_bytes(struct.pack('>I', value & 0xFFFFFFFF))
    
    def write_s1(self, value: int) -> None:
        """Write a signed byte (1 byte)"""
        self._write_bytes(struct.pack('>b', value))
    
    def write_s2(self, value: int) -> None:
        """Write a signed short (2 bytes)"""
        self._write_bytes(struct.pack('>h', value))
    
    def write_s4(self, value: int) -> None:
        """Write a signed int (4 bytes)"""
        self._write_bytes(struct.pack('>i', value))
    
    def write_float(self, value: float) -> None:
        """Write a float (4 bytes)"""
        self._write_bytes(struct.pack('>f', value))
    
    def write_double(self, value: float) -> None:
        """Write a double (8 bytes)"""
        self._write_bytes(struct.pack('>d', value))
    
    def write_boolean(self, value: bool) -> None:
        """Write a boolean (1 byte)"""
        self._write_bytes(bytes([1 if value else 0]))
    
    def write_null(self) -> None:
        """Write a null value (0 bytes of content)"""
        pass
    
    def write_string(self, value: str) -> None:
        """Write a string with 1-byte length prefix"""
        encoded = value.encode('utf-8')
        length = len(encoded)
        if length > 255:
            raise ValueError("String too long for write_string (max 255 bytes)")
        self.write_u1(length)
        self._write_bytes(encoded)
    
    def write_long_string(self, value: str) -> None:
        """Write a string with 2-byte length prefix"""
        encoded = value.encode('utf-8')
        length = len(encoded)
        if length > 65535:
            raise ValueError("String too long for write_long_string (max 65535 bytes)")
        self.write_u2(length)
        self._write_bytes(encoded)
    
    def write_array(self, value: List[Any]) -> None:
        """Write an array with 1-byte size prefix"""
        if len(value) > 255:
            raise ValueError("Array too large for write_array (max 255 elements)")
        self.write_u1(len(value))
        for item in value:
            self.write_value(item)
    
    def write_long_array(self, value: List[Any]) -> None:
        """Write an array with 2-byte size prefix"""
        if len(value) > 65535:
            raise ValueError("Array too large for write_long_array (max 65535 elements)")
        self.write_u2(len(value))
        for item in value:
            self.write_value(item)
    
    def write_map(self, value: Dict[str, Any]) -> None:
        """Write a map with 1-byte size prefix"""
        if len(value) > 255:
            raise ValueError("Map too large for write_map (max 255 entries)")
        self.write_u1(len(value))
        for k, v in value.items():
            self.write_value(k)
            self.write_value(v)
    
    def write_long_map(self, value: Dict[str, Any]) -> None:
        """Write a map with 2-byte size prefix"""
        if len(value) > 65535:
            raise ValueError("Map too large for write_long_map (max 65535 entries)")
        self.write_u2(len(value))
        for k, v in value.items():
            self.write_value(k)
            self.write_value(v)
    
    def write_value(self, value: Any) -> None:
        """Write a value with appropriate type identifier"""
        if value is None:
            self.write_type(self.TYPE_NULL)
            self.write_null()
        elif isinstance(value, bool):
            self.write_type(self.TYPE_BOOLEAN)
            self.write_boolean(value)
        elif isinstance(value, int):
            if -128 <= value <= 127:
                self.write_type(self.TYPE_S1)
                self.write_s1(value)
            elif -32768 <= value <= 32767:
                self.write_type(self.TYPE_S2)
                self.write_s2(value)
            elif -2147483648 <= value <= 2147483647:
                self.write_type(self.TYPE_S4)
                self.write_s4(value)
            elif 0 <= value <= 255:
                self.write_type(self.TYPE_U1)
                self.write_u1(value)
            elif 0 <= value <= 65535:
                self.write_type(self.TYPE_U2)
                self.write_u2(value)
            elif 0 <= value <= 4294967295:
                self.write_type(self.TYPE_U4)
                self.write_u4(value)
            else:
                # Convert to double if integer is out of range
                self.write_type(self.TYPE_DOUBLE)
                self.write_double(float(value))
        elif isinstance(value, float):
            # Use double for better precision
            self.write_type(self.TYPE_DOUBLE)
            self.write_double(value)
        elif isinstance(value, str):
            encoded = value.encode('utf-8')
            if len(encoded) <= 255:
                self.write_type(self.TYPE_STRING)
                self.write_string(value)
            else:
                self.write_type(self.TYPE_LONG_STRING)
                self.write_long_string(value)
        elif isinstance(value, list):
            if len(value) <= 255:
                self.write_type(self.TYPE_ARRAY)
                self.write_array(value)
            else:
                self.write_type(self.TYPE_LONG_ARRAY)
                self.write_long_array(value)
        elif isinstance(value, dict):
            if len(value) <= 255:
                self.write_type(self.TYPE_MAP)
                self.write_map(value)
            else:
                self.write_type(self.TYPE_LONG_MAP)
                self.write_long_map(value)
        else:
            # Try to convert to string as fallback
            self.write_type(self.TYPE_STRING)
            self.write_string(str(value))
    
    def encode_json(self, data: Any) -> bytes:
        """Encode JSON data to binary format"""
        self.reset()
        self.write_value(data)
        return self.get_buffer()
    
    def decode_to_json(self) -> Any:
        """Decode binary format to JSON data"""
        return self.read_value()
    
    # Internal helper methods
    def _read_bytes(self, length: int) -> bytes:
        """Read specified number of bytes from the buffer"""
        self.buffer.seek(self.position)
        data = self.buffer.read(length)
        if len(data) < length:
            raise EOFError(f"Tried to read {length} bytes but only {len(data)} available")
        self.position += length
        return data
    
    def _write_bytes(self, data: bytes) -> None:
        """Write bytes to the buffer"""
        self.buffer.write(data)


class AgBinaryClient:
    """Client for making requests with application/x-ag-binary content type"""
    
    def __init__(self, base_url: str, default_headers: Dict[str, str] = None):
        self.base_url = base_url
        self.protocol = BinaryProtocol()
        self.default_headers = {
            'Content-Type': 'application/x-ag-binary',
            'Accept': 'application/x-ag-binary'
        }
        if default_headers:
            self.default_headers.update(default_headers)
    
    def request(self, method: str, path: str, data: Any = None, 
                headers: Dict[str, str] = None, params: Dict[str, str] = None) -> Tuple[int, Any]:
        """
        Make a request with the specified method, path, and data.
        
        Args:
            method: HTTP method (GET, POST, etc.)
            path: Path to append to base_url
            data: Data to send (will be encoded as binary)
            headers: Additional headers to send
            params: URL parameters
            
        Returns:
            Tuple of (status_code, decoded_response_data)
        """
        url = f"{self.base_url.rstrip('/')}/{path.lstrip('/')}"
        
        request_headers = self.default_headers.copy()
        if headers:
            request_headers.update(headers)
        
        binary_data = None
        if data is not None:
            binary_data = self.protocol.encode_json(data)
        
        response = requests.request(
            method=method,
            url=url,
            headers=request_headers,
            data=binary_data,
            params=params
        )
        
        status_code = response.status_code
        
        if response.content and response.headers.get('Content-Type') == 'application/x-ag-binary':
            self.protocol.set_buffer(response.content)
            # Save the buffer to a file
            with open('response_buffer.bin', 'wb') as f:
                f.write(self.protocol.get_buffer())
            decoded_data = self.protocol.decode_to_json()
            return status_code, decoded_data
        
        return status_code, response.content
    
    def get(self, path: str, headers: Dict[str, str] = None, params: Dict[str, str] = None) -> Tuple[int, Any]:
        """Make a GET request"""
        return self.request('GET', path, headers=headers, params=params)
    
    def post(self, path: str, data: Any, 
             headers: Dict[str, str] = None, params: Dict[str, str] = None) -> Tuple[int, Any]:
        """Make a POST request"""
        return self.request('POST', path, data, headers=headers, params=params)
    
    def put(self, path: str, data: Any, 
            headers: Dict[str, str] = None, params: Dict[str, str] = None) -> Tuple[int, Any]:
        """Make a PUT request"""
        return self.request('PUT', path, data, headers=headers, params=params)
    
    def delete(self, path: str, data: Any = None, 
               headers: Dict[str, str] = None, params: Dict[str, str] = None) -> Tuple[int, Any]:
        """Make a DELETE request"""
        return self.request('DELETE', path, data, headers=headers, params=params)