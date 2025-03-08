#pragma once
#include <cstdint>
#include <string>
#include <vector>
#include <optional>
#include <sstream>
#include <chrono>
#include <type_traits>
#include <stdexcept>
#include <utils/macros.hpp>
#include <queue>
#include <unordered_set>
#include <logging/logger.hpp>

namespace hydra
{
    // Forward declarations
    class Map;
    class List;
    class ValueVariant;
    class Request;

    enum class ValueType : uint8_t
    {
        Integer = 0,
        Double = 1,
        Boolean = 2,
        String = 3,
        Map = 4,
        List = 5,
        DateTime = 6,
        HiResDateTime = 7,
        Binary = 8,
        Compressed = 9,
    };

    class Value
    {
    private:
        void **vftable;

    public:
        virtual void dtor() = 0;
        virtual ValueType type() = 0;
        std::string get_type_name();
    };

    class IntegerValue : public Value
    {
    public:
        SET_MEMBER_OFFSET(int64_t, value, 0x8);

        virtual void dtor() override;
        virtual ValueType type() override;
        std::string to_string();
    };

    class DoubleValue : public Value
    {
    public:
        double value;

        virtual void dtor() override;
        virtual ValueType type() override;
        std::string to_string();
    };

    class BooleanValue : public Value
    {
    public:
        bool value;

        virtual void dtor() override;
        virtual ValueType type() override;
        std::string to_string();
    };

    class StringValue : public Value
    {
    public:
        std::string value;

        virtual void dtor() override;
        virtual ValueType type() override;
        std::string to_string();
    };

    struct MapEntry
    {
        MapEntry *left_child;
        void* padding01;
        MapEntry *right_child;
        bool padding02;
        bool is_last;
        StringValue *key;
        Value *value;
    };

    class ValueVariant;

    class MapValue : public Value
    {
    public:
        MapEntry *root;
        MapEntry *sentinel;
        int32_t num_entries;

        Value *get_value_by_key(const std::string &key_str)
        {
            MapEntry *map_data = *reinterpret_cast<MapEntry **>(reinterpret_cast<uintptr_t>(this) + 0x8);
            MapEntry *map_data_1 = *reinterpret_cast<MapEntry **>(reinterpret_cast<uintptr_t>(map_data) + 0x8);

            while (!*reinterpret_cast<bool *>(reinterpret_cast<uintptr_t>(map_data_1) + 0x19))
            {
                StringValue *cur_key = *reinterpret_cast<StringValue **>(reinterpret_cast<uintptr_t>(map_data_1) + 0x20);
                if (cur_key->to_string() == key_str)
                {
                    map_data_1 = *reinterpret_cast<MapEntry **>(reinterpret_cast<uintptr_t>(map_data_1) + 0x10);
                }
                else
                {
                    map_data = map_data_1;
                    map_data_1 = *reinterpret_cast<MapEntry **>(reinterpret_cast<uintptr_t>(map_data_1));
                }
            }

            if (map_data == *reinterpret_cast<MapEntry **>(reinterpret_cast<uintptr_t>(this) + 0x8))
            {
                return nullptr;
            }

            StringValue *found_key = *reinterpret_cast<StringValue **>(reinterpret_cast<uintptr_t>(map_data) + 0x20);
            if (found_key->to_string() != key_str)
            {
                return nullptr;
            }

            return *reinterpret_cast<Value **>(reinterpret_cast<uintptr_t>(map_data) + 0x28);
        }
    };

    class MapIterator
    {
    private:
        std::vector<std::pair<std::string, Value *>> m_entries;
        size_t m_index;

    public:
        MapIterator(MapValue *map, bool begin = true)
        {
            m_index = 0;

            if (!map || !begin)
            {
                return; // Empty iterator or end iterator
            }

            MapEntry *sentinel = *reinterpret_cast<MapEntry **>(reinterpret_cast<uintptr_t>(map) + 0x8);
            if (!sentinel)
            {
                return;
            }

            MapEntry *first_node = *reinterpret_cast<MapEntry **>(reinterpret_cast<uintptr_t>(sentinel) + 0x8);
            if (!first_node || first_node == sentinel)
            {
                return;
            }

            std::queue<MapEntry *> queue;
            std::unordered_set<MapEntry *> visited;

            queue.push(first_node);
            visited.insert(first_node);

            while (!queue.empty())
            {
                MapEntry *node = queue.front();
                queue.pop();

                if (node == sentinel)
                {
                    continue;
                }

                // Extract key-value pair
                StringValue *key = *reinterpret_cast<StringValue **>(reinterpret_cast<uintptr_t>(node) + 0x20);
                Value *value = *reinterpret_cast<Value **>(reinterpret_cast<uintptr_t>(node) + 0x28);

                m_entries.push_back({key->to_string(), value});

                // Add unvisited children to queue
                bool is_last = *reinterpret_cast<bool *>(reinterpret_cast<uintptr_t>(node) + 0x19);
                if (!is_last)
                {
                    MapEntry *left = *reinterpret_cast<MapEntry **>(reinterpret_cast<uintptr_t>(node) + 0x00);
                    if (left && left != sentinel && visited.find(left) == visited.end())
                    {
                        queue.push(left);
                        visited.insert(left);
                    }

                    MapEntry *right = *reinterpret_cast<MapEntry **>(reinterpret_cast<uintptr_t>(node) + 0x10);
                    if (right && right != sentinel && visited.find(right) == visited.end())
                    {
                        queue.push(right);
                        visited.insert(right);
                    }
                }
            }
        }

        // Return a pair of string and ValueVariant
        std::pair<std::string, ValueVariant> operator*() const;

        MapIterator &operator++()
        {
            if (m_index < m_entries.size())
            {
                ++m_index;
            }
            return *this;
        }

        MapIterator operator++(int)
        {
            MapIterator tmp = *this;
            ++(*this);
            return tmp;
        }

        bool operator==(const MapIterator &other) const
        {
            if (m_index >= m_entries.size() && other.m_index >= other.m_entries.size())
            {
                return true;
            }
            return m_index == other.m_index && m_entries.size() == other.m_entries.size();
        }

        bool operator!=(const MapIterator &other) const
        {
            return !(*this == other);
        }

        std::string key() const
        {
            if (m_index >= m_entries.size())
            {
                throw std::runtime_error("Accessing key of invalid iterator");
            }
            return m_entries[m_index].first;
        }

        ValueVariant value() const;
    };

    class IterableMap : public MapValue
    {
    public:
        MapIterator begin()
        {
            return MapIterator(this, true);
        }

        MapIterator end()
        {
            return MapIterator(this, false);
        }
        
        template <typename T>
        std::optional<T> get(const std::string &key);

        bool contains(const std::string &key)
        {
            // return get_value_by_key(key) != nullptr;
            return false;
        }
    };

    class ListValue;

    class List
    {
    public:
        std::vector<Value *> values;

        size_t size() const;
        bool empty() const;

        ValueVariant at(size_t index) const;

        class Iterator
        {
        private:
            const std::vector<Value *> *m_values;
            size_t m_index;

        public:
            Iterator(const std::vector<Value *> *values, size_t index = 0);

            ValueVariant operator*() const;

            Iterator &operator++();

            Iterator operator++(int);

            bool operator==(const Iterator &other) const;
            bool operator!=(const Iterator &other) const;
        };

        Iterator begin() const;
        Iterator end() const;

        template <typename T>
        std::vector<T> to_vector() const;
    };

    class ListValue : public Value
    {
    public:
        SET_MEMBER_OFFSET_REFERENCE(List, list, 0x8);

        virtual void dtor() override;
        virtual ValueType type() override;

        const List* get_list() const;

        // Convenience methods that delegate to the List
        List::Iterator begin() const;
        List::Iterator end() const;
        size_t size() const;
        ValueVariant at(size_t index) const;

        // Convert to string for debugging/display
        std::string to_string() const;

        // Helper to check if all elements in the list are of a specific type
        template <typename T>
        bool all_of_type() const;

        // Convert to vector if all elements are of the same type
        template <typename T>
        std::optional<std::vector<T>> as_vector() const;
    };

    // Value wrapper class
    class ValueVariant
    {
    private:
        Value *m_value;

    public:
        ValueVariant(Value *value);

        // Check if the value is of a specific type
        template <typename T>
        bool is() const;

        // Get the value as a specific type (with type checking)
        template <typename T>
        T as() const;

        // Get the raw Value pointer
        Value *get() const;

        // Get the type
        ValueType type() const;

        // Helper for converting to string (for debugging/logging)
        std::string to_string() const;
    };

    namespace ValueUtils
    {
        // Generic value printer that handles all value types recursively
        void print_value(const ValueVariant &value, const std::string &prefix = "", int indent_level = 0);

        // Print a request's data field with proper formatting
        void log_request_data(const Request *request);
    }

// Include template implementations at the end of the header
#include "value.inl"
}