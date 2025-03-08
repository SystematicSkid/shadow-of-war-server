#pragma once
#include <string>
#include <utils/macros.hpp>
#include "value.hpp"

namespace hydra
{
    class String : public Value
    {
    public:
        SET_MEMBER_OFFSET( std::string, string, 0x8 );

        bool empty()
        {
            return this->string.empty();
        }

        std::string to_string()
        {
            return this->string;
        }
    };
}
