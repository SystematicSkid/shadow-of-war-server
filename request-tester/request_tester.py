import json
from binary_protocol import AgBinaryClient

def test_client():
    """Test the AG Binary client"""
    # Initialize client
    client = AgBinaryClient(
        base_url="https://kraken-api.wbagora.com",
        default_headers={
            "x-hydra-api-key": "864a45396e0146d1bdd093dc0ef12ab0",
            "Content-Type": "application/x-ag-binary",
            "x-hydra-user-agent": "Hydra-Cpp/1.50.1",
            #"x-hydra-access-token": "11111111111111111111111",
            "accept-language": "enUS"
        }
    )
    
    request_data = {
        "fail_on_missing": 0,
        "steam": "11111111111111111111111"
    }
    
    # Make a POST request
    try:
        status_code, response_data = client.post(
            path="/auth",
            data=request_data,
        )
        print(f"Status: {status_code}")
        print("Response:", json.dumps(response_data, indent=2))
    except Exception as e:
        print(f"Request failed: {e}")
        
    print("\nRequest would be sent with these properties:")
    print(f"- URL: {client.base_url}/auth")
    print(f"- Headers: {client.default_headers}")
    print(f"- Data: {request_data}")

if __name__ == "__main__":
    print("\n===== Testing AG Binary Client =====")
    test_client()