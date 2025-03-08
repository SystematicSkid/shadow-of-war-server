#include "hydra/value.hpp"
#include <logging/logger.hpp>
#include <hydra/request.hpp>

namespace hydra
{
    std::string Value::get_type_name()
    {
        switch (type())
        {
        case ValueType::Integer:
            return "Integer";
        case ValueType::Double:
            return "Double";
        case ValueType::Boolean:
            return "Boolean";
        case ValueType::String:
            return "String";
        case ValueType::Map:
            return "Map";
        case ValueType::List:
            return "List";
        case ValueType::DateTime:
            return "DateTime";
        case ValueType::HiResDateTime:
            return "HiResDateTime";
        case ValueType::Binary:
            return "Binary";
        case ValueType::Compressed:
            return "Compressed";
        default:
            return "Unknown";
        }
    }

    void IntegerValue::dtor() {}

    ValueType IntegerValue::type()
    {
        return ValueType::Integer;
    }

    std::string IntegerValue::to_string()
    {
        return std::to_string(value);
    }

    void DoubleValue::dtor() {}

    ValueType DoubleValue::type()
    {
        return ValueType::Double;
    }

    std::string DoubleValue::to_string()
    {
        return std::to_string(value);
    }

    void BooleanValue::dtor() {}

    ValueType BooleanValue::type()
    {
        return ValueType::Boolean;
    }

    std::string BooleanValue::to_string()
    {
        return value ? "true" : "false";
    }

    void StringValue::dtor() {}

    ValueType StringValue::type()
    {
        return ValueType::String;
    }

    std::string StringValue::to_string()
    {
        return value;
    }

    size_t List::size() const
    {
        return values.size();
    }

    bool List::empty() const
    {
        return values.empty();
    }

    ValueVariant List::at(size_t index) const
    {
        if (index >= values.size())
        {
            throw std::out_of_range("List index out of range");
        }
        return ValueVariant(values[index]);
    }

    List::Iterator::Iterator(const std::vector<Value *> *values, size_t index)
        : m_values(values), m_index(index) {}

    ValueVariant List::Iterator::operator*() const
    {
        if (!m_values || m_index >= m_values->size())
        {
            throw std::runtime_error("Dereferencing invalid iterator");
        }
        return ValueVariant((*m_values)[m_index]);
    }

    List::Iterator &List::Iterator::operator++()
    {
        if (m_values && m_index < m_values->size())
        {
            ++m_index;
        }
        return *this;
    }

    List::Iterator List::Iterator::operator++(int)
    {
        Iterator tmp = *this;
        ++(*this);
        return tmp;
    }

    bool List::Iterator::operator==(const Iterator &other) const
    {
        // Both at end
        if ((!m_values || m_index >= m_values->size()) &&
            (!other.m_values || other.m_index >= other.m_values->size()))
        {
            return true;
        }
        return m_values == other.m_values && m_index == other.m_index;
    }

    bool List::Iterator::operator!=(const Iterator &other) const
    {
        return !(*this == other);
    }

    List::Iterator List::begin() const
    {
        return Iterator(&values, 0);
    }

    List::Iterator List::end() const
    {
        return Iterator(&values, values.size());
    }

    // ListValue implementation
    void ListValue::dtor() {}

    ValueType ListValue::type()
    {
        return ValueType::List;
    }

    const List* ListValue::get_list() const
    {
        return list;
    }

    List::Iterator ListValue::begin() const
    {
        return list->begin();
    }

    List::Iterator ListValue::end() const
    {
        return list->end();
    }

    size_t ListValue::size() const
    {
        return list->size();
    }

    ValueVariant ListValue::at(size_t index) const
    {
        return list->at(index);
    }

    std::string ListValue::to_string() const
    {
        std::ostringstream result;
        result << "[";

        bool first = true;
        for (auto it = list->begin(); it != list->end(); ++it)
        {
            if (!first)
            {
                result << ", ";
            }
            first = false;

            ValueVariant value = *it;
            result << value.to_string();
        }

        result << "]";
        return result.str();
    }

    ValueVariant::ValueVariant(Value *value) : m_value(value) {}

    Value *ValueVariant::get() const
    {
        return m_value;
    }

    ValueType ValueVariant::type() const
    {
        return m_value ? m_value->type() : static_cast<ValueType>(0xFF);
    }

    std::string ValueVariant::to_string() const
    {
        if (!m_value)
            return "null";

        switch (m_value->type())
        {
        case ValueType::Integer:
            return std::to_string(as<int64_t>());
        case ValueType::Double:
            return std::to_string(as<double>());
        case ValueType::Boolean:
            return as<bool>() ? "true" : "false";
        case ValueType::String:
            return as<std::string>();
        case ValueType::Map:
            return "[Map]";
        case ValueType::List:
            return as<ListValue *>()->to_string();
        case ValueType::DateTime:
        case ValueType::HiResDateTime:
            return "[DateTime]";
        case ValueType::Binary:
            return "[Binary data]";
        case ValueType::Compressed:
            return "[Compressed data]";
        default:
            return "[Unknown]";
        }
    }

    std::pair<std::string, ValueVariant> MapIterator::operator*() const
    {
        if (m_index >= m_entries.size())
        {
            throw std::runtime_error("Dereferencing invalid iterator");
        }
        return {m_entries[m_index].first, ValueVariant(m_entries[m_index].second)};
    }

    ValueVariant MapIterator::value() const
    {
        if (m_index >= m_entries.size())
        {
            throw std::runtime_error("Accessing value of invalid iterator");
        }
        return ValueVariant(m_entries[m_index].second);
    }

    void ValueUtils::log_request_data(const Request *request)
    {
        if (!request)
        {
            LOG_INFO("Null request");
            return;
        }

        LOG_INFO("Response: {}", request->endpoint);
        LOG_INFO("Response Code: {}", request->response_code);

        if (!request->data)
        {
            LOG_INFO("No data");
            return;
        }

        ValueVariant data = request->get_data();
        print_value(data, "Data: ");
    }

    void ValueUtils::print_value(const ValueVariant &value, const std::string &prefix, int indent_level)
    {
        std::string indent(indent_level * 2, ' ');

        switch (value.type())
        {
        case ValueType::Integer:
            LOG_INFO("{}{}Integer: {}", indent, prefix, value.as<int64_t>());
            break;

        case ValueType::Double:
            LOG_INFO("{}{}Double: {}", indent, prefix, value.as<double>());
            break;

        case ValueType::Boolean:
            LOG_INFO("{}{}Boolean: {}", indent, prefix, value.as<bool>() ? "true" : "false");
            break;

        case ValueType::String:
            LOG_INFO("{}{}String: '{}'", indent, prefix, value.as<std::string>());
            break;

        case ValueType::Map:
        {
            LOG_INFO("{}{}Map:", indent, prefix);
            MapValue *mapValue = static_cast<MapValue *>(value.get());
            if (mapValue)
            {
                IterableMap *iterableMap = static_cast<IterableMap *>(mapValue);
                if (iterableMap)
                {
                    for (auto it = iterableMap->begin(); it != iterableMap->end(); ++it)
                    {
                        auto [key, subValue] = *it;
                        print_value(subValue, std::format("'{}' => ", key), indent_level + 1);
                    }
                }
            }
            break;
        }

        case ValueType::List:
        {
            ListValue *listValue = static_cast<ListValue *>(value.get());
            LOG_INFO("{}{}List with {} items:", indent, prefix, listValue->size());

            int index = 0;
            for (auto it = listValue->begin(); it != listValue->end(); ++it, ++index)
            {
                print_value(*it, std::format("[{}]: ", index), indent_level + 1);
            }
            break;
        }

        case ValueType::DateTime:
        case ValueType::HiResDateTime:
            LOG_INFO("{}{}DateTime: {}", indent, prefix, value.to_string());
            break;

        case ValueType::Binary:
            LOG_INFO("{}{}Binary data", indent, prefix);
            break;

        case ValueType::Compressed:
            LOG_INFO("{}{}Compressed data", indent, prefix);
            break;

        default:
            LOG_INFO("{}{}Unknown type", indent, prefix);
            break;
        }
    }
}