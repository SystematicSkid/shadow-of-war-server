#pragma once
#include <filesystem>
#include <fstream>
#include <sstream>
#include <iomanip>
#include <ctime>
#include <format>
#include <regex>
#include <hydra/value.hpp>
#include <hydra/request.hpp>
#include <hydra/client.hpp>

namespace fs = std::filesystem;
using namespace hydra;

class RequestFileLogger {
private:
    bool enabled = false;
    std::string base_dir;
    
    struct UrlComponents {
        std::string domain;
        std::string path;
        std::string query;
    };
    
    UrlComponents parse_url(const std::string& host, const std::string& endpoint, bool is_full_url = false) {
        UrlComponents result;
        
        if (is_full_url) {
            size_t protocol_pos = endpoint.find("://");
            if (protocol_pos != std::string::npos) {
                size_t domain_start = protocol_pos + 3;
                size_t path_start = endpoint.find('/', domain_start);
                
                if (path_start != std::string::npos) {
                    result.domain = sanitize_filename(endpoint.substr(domain_start, path_start - domain_start));
                    std::string path_part = endpoint.substr(path_start);
                    
                    size_t query_pos = path_part.find('?');
                    if (query_pos != std::string::npos) {
                        result.path = path_part.substr(0, query_pos);
                        result.query = path_part.substr(query_pos + 1);
                    } else {
                        result.path = path_part;
                        result.query = "";
                    }
                } else {
                    result.domain = sanitize_filename(endpoint.substr(domain_start));
                    result.path = "";
                    result.query = "";
                }
            } else {
                result.domain = "unknown";
                result.path = endpoint;
                result.query = "";
            }
        } else {
            result.domain = sanitize_filename(host);
            
            size_t query_pos = endpoint.find('?');
            if (query_pos != std::string::npos) {
                result.path = endpoint.substr(0, query_pos);
                result.query = endpoint.substr(query_pos + 1);
            } else {
                result.path = endpoint;
                result.query = "";
            }
        }
        
        if (!result.path.empty() && result.path[0] == '/') {
            result.path = result.path.substr(1);
        }
        
        return result;
    }

    std::string normalize_domain(const std::string& domain) {
        std::string result = domain;
        
        const std::vector<std::string> prefixes = {
            "https___", "http___", "wss___", "ws___", 
            "https_", "http_", "wss_", "ws_"
        };
        
        for (const auto& prefix : prefixes) {
            if (result.size() >= prefix.size() && result.substr(0, prefix.size()) == prefix) {
                result = result.substr(prefix.size());
                break;
            }
        }
        
        return result;
    }

    
    std::string create_directory_structure(const UrlComponents& url_components) {
        std::string full_path = base_dir;
        
        std::string normalized_domain = normalize_domain(url_components.domain);
        full_path += "/" + normalized_domain;
        ensure_directory_exists(full_path);
        
        std::string path_copy = url_components.path;
        std::vector<std::string> segments;
        
        size_t pos = 0;
        while ((pos = path_copy.find('/')) != std::string::npos) {
            std::string segment = path_copy.substr(0, pos);
            if (!segment.empty()) {
                segments.push_back(sanitize_filename(segment));
            }
            path_copy.erase(0, pos + 1);
        }
        
        if (!path_copy.empty()) {
            segments.push_back(sanitize_filename(path_copy));
        }
        
        for (const auto& segment : segments) {
            full_path += "/" + segment;
            ensure_directory_exists(full_path);
        }
        
        if (!url_components.query.empty()) {
            std::string query_dir = "query_" + sanitize_filename(url_components.query);
            if (query_dir.length() > 100) {
                query_dir = "query_" + std::to_string(std::hash<std::string>{}(url_components.query));
            }
            full_path += "/" + query_dir;
            ensure_directory_exists(full_path);
        }
        
        return full_path;
    }
    
    std::string sanitize_filename(const std::string& filename) {
        std::string result = filename;
        
        const std::string invalid_chars = "\\/:?\"<>|*&=#%+; ";
        for (char& c : result) {
            if (invalid_chars.find(c) != std::string::npos) {
                c = '_';
            }
        }
        
        if (result.length() > 100) {
            result = result.substr(0, 100);
        }
        
        return result;
    }
    
    void ensure_directory_exists(const std::string& dir_path) {
        if (!fs::exists(dir_path)) {
            try {
                fs::create_directories(dir_path);
            } catch (const std::exception& e) {
                LOG_ERROR("Failed to create directory '{}': {}", dir_path, e.what());
            }
        }
    }
    
    std::string get_timestamp() {
        auto now = std::chrono::system_clock::now();
        auto time = std::chrono::system_clock::to_time_t(now);
        auto ms = std::chrono::duration_cast<std::chrono::milliseconds>(
            now.time_since_epoch()) % 1000;
            
        std::stringstream ss;
        ss << std::put_time(std::localtime(&time), "%Y%m%d_%H%M%S_") 
           << std::setfill('0') << std::setw(3) << ms.count();
        return ss.str();
    }
    
    void value_to_string(const ValueVariant& value, std::stringstream& ss, 
                          const std::string& prefix = "", int indent_level = 0) {
        std::string indent(indent_level * 2, ' ');

        switch (value.type()) {
        case ValueType::Integer:
            ss << indent << prefix << "Integer: " << value.as<int64_t>() << "\n";
            break;

        case ValueType::Double:
            ss << indent << prefix << "Double: " << value.as<double>() << "\n";
            break;

        case ValueType::Boolean:
            ss << indent << prefix << "Boolean: " << (value.as<bool>() ? "true" : "false") << "\n";
            break;

        case ValueType::String:
            ss << indent << prefix << "String: '" << value.as<std::string>() << "'\n";
            break;

        case ValueType::Map:
        {
            ss << indent << prefix << "Map:\n";
            MapValue* mapValue = static_cast<MapValue*>(value.get());
            if (mapValue) {
                IterableMap* iterableMap = static_cast<IterableMap*>(mapValue);
                if (iterableMap) {
                    for (auto it = iterableMap->begin(); it != iterableMap->end(); ++it) {
                        auto [key, subValue] = *it;
                        value_to_string(subValue, ss, std::format("'{}' => ", key), indent_level + 1);
                    }
                }
            }
            break;
        }

        case ValueType::List:
        {
            ListValue* listValue = static_cast<ListValue*>(value.get());
            ss << indent << prefix << "List with " << listValue->size() << " items:\n";

            int index = 0;
            for (auto it = listValue->begin(); it != listValue->end(); ++it, ++index) {
                value_to_string(*it, ss, std::format("[{}]: ", index), indent_level + 1);
            }
            break;
        }

        case ValueType::DateTime:
        case ValueType::HiResDateTime:
            ss << indent << prefix << "DateTime: " << value.to_string() << "\n";
            break;

        case ValueType::Binary:
            ss << indent << prefix << "Binary data\n";
            break;

        case ValueType::Compressed:
            ss << indent << prefix << "Compressed data\n";
            break;

        default:
            ss << indent << prefix << "Unknown type\n";
            break;
        }
    }

public:
    RequestFileLogger(const std::string& directory = "request_logs") : base_dir(directory) {}
    
    void enable_logging(bool enable) {
        enabled = enable;
    }
    
    void set_base_directory(const std::string& directory) {
        base_dir = directory;
    }
    
    bool is_enabled() const {
        return enabled;
    }
    
    void save_request(hydra::Client* client, const std::string& endpoint, const std::string& method, 
                       hydra::Value* data) {
        if (!enabled || !client) return;
        
        try {
            ensure_directory_exists(base_dir);
            
            std::string host = client->host_address;
            
            UrlComponents url_components = parse_url(host, endpoint);
            
            std::string dir_path = create_directory_structure(url_components);
            
            std::string filename = get_timestamp() + "_" + method + "_request.txt";
            std::string filepath = dir_path + "/" + filename;
            
            std::ofstream file(filepath);
            if (!file.is_open()) {
                LOG_ERROR("Failed to open file for writing: {}", filepath);
                return;
            }
            
            file << "Host: " << host << "\n";
            file << "Endpoint: " << endpoint << "\n";
            file << "Method: " << method << "\n";
            file << "Timestamp: " << get_timestamp() << "\n";
            
            if (!url_components.query.empty()) {
                file << "Query Parameters: " << url_components.query << "\n";
            }
            
            file << "\n";
            
            if (data) {
                file << "Request Data:\n";
                std::stringstream ss;
                ValueVariant value_var(data);
                value_to_string(value_var, ss);
                file << ss.str();
            } else {
                file << "No request data\n";
            }
            
            file.close();
            LOG_INFO("Request saved to file: {}", filepath);
        } catch (const std::exception& e) {
            LOG_ERROR("Error saving request to file: {}", e.what());
        }
    }
    
    void save_response(hydra::Client* client, const hydra::Request* request) {
        if (!enabled || !request) return;
        
        try {
            ensure_directory_exists(base_dir);
            
            UrlComponents url_components = parse_url("", request->endpoint, true);
            
            std::string dir_path = create_directory_structure(url_components);
            
            std::string filename = get_timestamp() + "_response_" + 
                                  std::to_string(request->response_code) + ".txt";
            std::string filepath = dir_path + "/" + filename;
            
            std::ofstream file(filepath);
            if (!file.is_open()) {
                LOG_ERROR("Failed to open file for writing: {}", filepath);
                return;
            }
            
            file << "Full URL: " << request->endpoint << "\n";
            file << "Response Code: " << request->response_code << "\n";
            file << "Timestamp: " << get_timestamp() << "\n";
            
            if (!url_components.query.empty()) {
                file << "Query Parameters: " << url_components.query << "\n";
            }
            
            file << "\n";
            
            if (request->data) {
                file << "Response Data:\n";
                std::stringstream ss;
                ValueVariant data = request->get_data();
                value_to_string(data, ss);
                file << ss.str();
            } else {
                file << "No response data\n";
            }
            
            file.close();
            LOG_INFO("Response saved to file: {}", filepath);
        } catch (const std::exception& e) {
            LOG_ERROR("Error saving response to file: {}", e.what());
        }
    }
};