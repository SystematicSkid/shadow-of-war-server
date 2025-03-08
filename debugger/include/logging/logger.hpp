#pragma once

#include <format>
#include <source_location>
#include <string_view>
#include <concepts>
#include <chrono>

namespace core::logging {

enum class LogLevel {
    Trace,  
    Debug,  
    Info,   
    Warning,
    Error,  
    Fatal   
};

class Logger;
Logger& getLogger();

class Logger {
public:
    friend Logger& getLogger();

    template<typename... Args>
    void log(LogLevel level,
             const std::source_location& loc,
             std::format_string<Args...> fmt,
             Args&&... args) {
        std::string message = std::format(fmt, std::forward<Args>(args)...);
        
        auto now = std::chrono::system_clock::now();
        auto time = std::chrono::system_clock::to_time_t(now);
        
        logImpl(level, message, loc);
    }

    void log(LogLevel level,
             const std::source_location& loc,
             std::string_view msg) {
        logImpl(level, std::string(msg), loc);
    }

private:
    Logger() = default;
    
    void logImpl(LogLevel level, 
                 const std::string& message,
                 const std::source_location& loc);
};

}

// Logging macros with cross-compiler compatibility
#ifdef ENABLE_LOGGING
    // Helper macro to handle the empty __VA_ARGS__ case
    #define LOG_INTERNAL(level, fmt, ...) \
        ::core::logging::getLogger().log(level, std::source_location::current(), fmt, ##__VA_ARGS__)

    #define LOG_TRACE(fmt, ...) LOG_INTERNAL(::core::logging::LogLevel::Trace, fmt, ##__VA_ARGS__)
    #define LOG_DEBUG(fmt, ...) LOG_INTERNAL(::core::logging::LogLevel::Debug, fmt, ##__VA_ARGS__)
    #define LOG_INFO(fmt, ...)  LOG_INTERNAL(::core::logging::LogLevel::Info, fmt, ##__VA_ARGS__)
    #define LOG_WARN(fmt, ...)  LOG_INTERNAL(::core::logging::LogLevel::Warning, fmt, ##__VA_ARGS__)
    #define LOG_ERROR(fmt, ...) LOG_INTERNAL(::core::logging::LogLevel::Error, fmt, ##__VA_ARGS__)
    #define LOG_FATAL(fmt, ...) LOG_INTERNAL(::core::logging::LogLevel::Fatal, fmt, ##__VA_ARGS__)
#else
    #define LOG_TRACE(fmt, ...)
    #define LOG_DEBUG(fmt, ...)
    #define LOG_INFO(fmt, ...)
    #define LOG_WARN(fmt, ...)
    #define LOG_ERROR(fmt, ...)
    #define LOG_FATAL(fmt, ...)
#endif