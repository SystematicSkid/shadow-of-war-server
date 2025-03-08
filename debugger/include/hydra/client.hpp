#pragma once
#include <utils/macros.hpp>
#include <string>
namespace hydra
{
    class Client
    {
    public:
        SET_MEMBER_OFFSET( std::string, host_address, 0x30 );
    };
}

