#pragma once
#include <string>
#include <utils/macros.hpp>
#include <hydra/value.hpp>

namespace hydra
{
    class Request
    {
    public:
        SET_MEMBER_OFFSET( std::string, endpoint, 0x20 );
        SET_MEMBER_OFFSET( std::string, content_type, 0x68 );
        SET_MEMBER_OFFSET( int32_t, response_code, 0xFC );
        SET_MEMBER_OFFSET( Value *, data, 0x120 );

        ValueVariant get_data() const 
        {
            return ValueVariant(data);
        }
    };
}
