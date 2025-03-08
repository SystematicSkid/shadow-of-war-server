#pragma once
#include <windows.h>
#include <cstdint>

#define STR_MERGE_IMPL(a, b) a##b
#define STR_MERGE(a, b) STR_MERGE_IMPL(a, b)

#define EXPAND(...) __VA_ARGS__
#define CONCAT(x, y) x##y
#define CONCAT_1(arg1, arg2) CONCAT_2(arg1, arg2)
#define MAKE_PAD(size) char CONCAT_1(pad_, __COUNTER__)[size]

#define PROPERTY(type, name) __declspec(property(get = property_get_##name, put = property_put_##name)) type name
#define PROPERTY_GET(type, name) inline type property_get_##name() const
#define PROPERTY_PUT(type, name) inline void property_put_##name(type val)


#define SET_MEMBER_OFFSET(type, name, offset)                                         \
    PROPERTY(type, name);                                                             \
    PROPERTY_GET(type, name)                                                          \
    {                                                                                 \
        return *reinterpret_cast<type *>(reinterpret_cast<uintptr_t>(this) + offset); \
    }                                                                                 \
    PROPERTY_PUT(type, name)                                                          \
    {                                                                                 \
        *reinterpret_cast<type *>(reinterpret_cast<uintptr_t>(this) + offset) = val;  \
    }

#define SET_MEMBER_OFFSET_REFERENCE(type, name, offset) \
    PROPERTY(type *, name);                            \
    PROPERTY_GET(type *, name)                         \
    {                                                  \
        return reinterpret_cast<type *>(reinterpret_cast<uintptr_t>(this) + offset); \
    }
