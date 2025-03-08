#include <Windows.h>
#include <logging/logger.hpp>
#include <minhook/MinHook.h>

#include <hydra/client.hpp>
#include <hydra/map.hpp>
#include <hydra/request.hpp>
#include <logging/request_logger.hpp>

RequestFileLogger g_file_logger;

typedef void *(*request_response_fn_t)(hydra::Client *client, void *unk, hydra::Request **request_ref);
PVOID original_request_response_fn = nullptr;
void *callback_request_response(hydra::Client *client, void *unk, hydra::Request **request_ref)
{
    LOG_INFO("Response: 0x{:X}", (uintptr_t)request_ref);
    hydra::Request *request = *request_ref;
    if (request_ref && *request_ref)
    {
        // Log request information with clean formatting
        hydra::ValueUtils::log_request_data(*request_ref);

        if (g_file_logger.is_enabled()) {
            g_file_logger.save_response(client, request);
        }
    }
    else
    {
        LOG_INFO("Invalid request reference");
    }

    printf("\n\n");
    return ((request_response_fn_t)original_request_response_fn)(client, unk, request_ref);
}

typedef void *(*make_request_fn_t)(hydra::Client *client, std::string &endpoint, std::string &method, hydra::MapValue *data, void *a5, void *a6, void *a7);

PVOID original_make_request_fn;
void *callback_make_request(hydra::Client *client, std::string &endpoint, std::string &method, hydra::MapValue *data, void *a5, void *a6, void *callback)
{
    LOG_INFO("Making {} request to {}", method, endpoint);
    hydra::ValueUtils::print_value( data );
    if (g_file_logger.is_enabled()) {
        g_file_logger.save_request(client, endpoint, method, data);
    }
    return ((make_request_fn_t)original_make_request_fn)(client, endpoint, method, data, a5, a6, callback);
}

bool setup_hooks()
{
    if (MH_Initialize() != MH_OK)
    {
        LOG_ERROR("Failed to initialize MinHook");
        return false;
    }

    uintptr_t game_module = (uintptr_t)GetModuleHandleA(0);
    if (!game_module)
    {
        LOG_ERROR("Failed to get game module");
        return false;
    }

    uintptr_t make_request_fn = game_module + 0x95C0F0;
    if (MH_CreateHook((LPVOID)make_request_fn, (LPVOID)callback_make_request, (LPVOID *)&original_make_request_fn) != MH_OK)
    {
        LOG_ERROR("Failed to hook make_request_fn");
        return false;
    }

    uintptr_t handle_response_fn = game_module + 0x961170;
    if (MH_CreateHook((LPVOID)handle_response_fn, (LPVOID)callback_request_response, (LPVOID *)&original_request_response_fn) != MH_OK)
    {
        LOG_ERROR("Failed to hook handle_response_fn");
        return false;
    }

    return MH_EnableHook(MH_ALL_HOOKS) == MH_OK;
}

HANDLE hThread;
HMODULE hDebugModule;
/* Main thread */
DWORD WINAPI MainThread(LPVOID lpParam)
{
    /* Allocate console */
    AllocConsole();
    FILE *f;
    freopen_s(&f, "CONOUT$", "w", stdout);

    LOG_INFO("Main thread started");

    g_file_logger.enable_logging(true);
    g_file_logger.set_base_directory("request_logs");

    if (!setup_hooks())
    {
        LOG_ERROR("Failed to setup hooks");
        FreeLibraryAndExitThread(hDebugModule, 1);
    }

    while (true)
    {
        Sleep(1000);
    }

    LOG_INFO("Goodbye!");
    FreeLibraryAndExitThread(hDebugModule, 0);
    return 0;
}

/* DllMain */
BOOL APIENTRY DllMain(HMODULE hModule, DWORD ul_reason_for_call, LPVOID lpReserved)
{
    if (ul_reason_for_call != DLL_PROCESS_ATTACH)
        return TRUE;

    hDebugModule = hModule;

    hThread = CreateThread(0, 0, MainThread, 0, 0, 0);
    if (hThread)
        CloseHandle(hThread);

    return TRUE;
}
