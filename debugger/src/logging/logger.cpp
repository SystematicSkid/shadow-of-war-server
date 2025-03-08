#include <logging/logger.hpp>
#include <iostream>
#include <mutex>

#ifdef _WIN32
#include <windows.h>
#endif

namespace core::logging
{

    namespace
    {
        bool initializeConsole()
        {
#ifdef _WIN32
            HANDLE hConsole = GetStdHandle(STD_OUTPUT_HANDLE);
            if (hConsole == INVALID_HANDLE_VALUE)
            {
                return false;
            }

            DWORD consoleMode;
            if (!GetConsoleMode(hConsole, &consoleMode))
            {
                return false;
            }

            consoleMode |= ENABLE_VIRTUAL_TERMINAL_PROCESSING;
            if (!SetConsoleMode(hConsole, consoleMode))
            {
                return false;
            }
#endif
            return true;
        }

        namespace Color
        {
            constexpr std::string_view Reset = "\033[0m";
            constexpr std::string_view Grey = "\033[38;2;150;150;150m";   // RGB grey
            constexpr std::string_view White = "\033[38;2;220;220;220m";  // Soft white
            constexpr std::string_view Green = "\033[38;2;100;200;100m";  // Softer green
            constexpr std::string_view Yellow = "\033[38;2;220;180;50m";  // Warm yellow
            constexpr std::string_view Red = "\033[38;2;220;100;100m";    // Soft red
            constexpr std::string_view Purple = "\033[38;2;180;100;180m"; // Soft purple

            constexpr std::string_view ErrorBg = "\033[48;2;40;0;0m";  // Dark red background
            constexpr std::string_view FatalBg = "\033[48;2;60;0;60m"; // Dark purple background

            static const std::string ErrorStyle = std::string(Red) + std::string(ErrorBg);
            static const std::string FatalStyle = std::string(Purple) + std::string(FatalBg);
        }

        std::string_view getLevelColor(LogLevel level)
        {
            switch (level)
            {
            case LogLevel::Trace:
                return Color::Grey;
            case LogLevel::Debug:
                return Color::White;
            case LogLevel::Info:
                return Color::Green;
            case LogLevel::Warning:
                return Color::Yellow;
            case LogLevel::Error:
                return Color::ErrorStyle;
            case LogLevel::Fatal:
                return Color::FatalStyle;
            default:
                return Color::White;
            }
        }

        std::string_view getLevelString(LogLevel level)
        {
            switch (level)
            {
            case LogLevel::Trace:
                return "TRACE";
            case LogLevel::Debug:
                return "DEBUG";
            case LogLevel::Info:
                return "INFO "; // Padding for alignment
            case LogLevel::Warning:
                return "WARN "; // Padding for alignment
            case LogLevel::Error:
                return "ERROR";
            case LogLevel::Fatal:
                return "FATAL";
            default:
                return "?????";
            }
        }

        std::string_view getFileName(std::string_view path)
        {
            size_t pos = path.find_last_of("/\\");
            return pos == std::string_view::npos ? path : path.substr(pos + 1);
        }
    }

    Logger &getLogger()
    {
        static Logger instance;
        static bool initialized = initializeConsole();
        return instance;
    }

    void Logger::logImpl(LogLevel level,
                         const std::string &message,
                         const std::source_location &loc)
    {
        static std::mutex logMutex;
        std::lock_guard<std::mutex> lock(logMutex);

        auto now = std::chrono::system_clock::now();
        auto timeT = std::chrono::system_clock::to_time_t(now);
        std::tm timeInfo;
#ifdef _WIN32
        localtime_s(&timeInfo, &timeT);
#else
        localtime_r(&timeT, &timeInfo);
#endif

        char timeBuffer[16];
        std::strftime(timeBuffer, sizeof(timeBuffer), "%H:%M:%S", &timeInfo);

        // [filename:line] HH:MM:SS LEVEL message
        std::cout << std::format("{}[{}:{}] {} [{}] {}{}\n",
                                 getLevelColor(level),
                                 getFileName(loc.file_name()),
                                 loc.line(),
                                 timeBuffer,
                                 getLevelString(level),
                                 message,
                                 Color::Reset);

        if (level >= LogLevel::Warning)
        {
            std::cout.flush();
        }
    }

}