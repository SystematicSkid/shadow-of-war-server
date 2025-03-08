// List template implementations
template <typename T>
std::vector<T> List::to_vector() const
{
    std::vector<T> result;
    result.reserve(values.size());

    for (const auto &value : values)
    {
        ValueVariant variant(value);
        if (variant.is<T>())
        {
            result.push_back(variant.as<T>());
        }
    }

    return result;
}

// ListValue template implementations
template <typename T>
bool ListValue::all_of_type() const
{
    for (auto it = list.begin(); it != list.end(); ++it)
    {
        ValueVariant value = *it;
        if (!value.is<T>())
        {
            return false;
        }
    }
    return true;
}

template <typename T>
std::optional<std::vector<T>> ListValue::as_vector() const
{
    if (!all_of_type<T>())
    {
        return std::nullopt;
    }

    std::vector<T> result;
    result.reserve(list.size());

    for (auto it = list.begin(); it != list.end(); ++it)
    {
        ValueVariant value = *it;
        result.push_back(value.as<T>());
    }

    return result;
}

// ValueVariant template implementations
template <typename T>
bool ValueVariant::is() const
{
    if (!m_value)
        return false;

    // Map ValueType to actual C++ type
    if constexpr (std::is_same_v<T, int64_t>)
    {
        return m_value->type() == ValueType::Integer;
    }
    else if constexpr (std::is_same_v<T, double>)
    {
        return m_value->type() == ValueType::Double;
    }
    else if constexpr (std::is_same_v<T, bool>)
    {
        return m_value->type() == ValueType::Boolean;
    }
    else if constexpr (std::is_same_v<T, std::string>)
    {
        return m_value->type() == ValueType::String;
    }
    else if constexpr (std::is_same_v<T, Map *>)
    {
        return m_value->type() == ValueType::Map;
    }
    else if constexpr (std::is_same_v<T, ListValue *>)
    {
        return m_value->type() == ValueType::List;
    }
    else if constexpr (std::is_same_v<T, std::chrono::system_clock::time_point>)
    {
        return m_value->type() == ValueType::DateTime ||
               m_value->type() == ValueType::HiResDateTime;
    }
    else if constexpr (std::is_same_v<T, std::vector<uint8_t>>)
    {
        return m_value->type() == ValueType::Binary;
    }

    return false;
}

template <typename T>
std::optional<T> IterableMap::get(const std::string &key)
{
    Value *value = get_value_by_key(key);
    if (!value)
    {
        return std::nullopt;
    }

    ValueVariant variant(value);
    if (!variant.is<T>())
    {
        return std::nullopt;
    }

    return variant.as<T>();
}

template <typename T>
T ValueVariant::as() const
{
    if (!is<T>())
    {
        throw std::bad_cast();
    }

    if constexpr (std::is_same_v<T, int64_t>)
    {
        return static_cast<IntegerValue *>(m_value)->value;
    }
    else if constexpr (std::is_same_v<T, double>)
    {
        return static_cast<DoubleValue *>(m_value)->value;
    }
    else if constexpr (std::is_same_v<T, bool>)
    {
        return static_cast<BooleanValue *>(m_value)->value;
    }
    else if constexpr (std::is_same_v<T, std::string>)
    {
        return static_cast<StringValue *>(m_value)->value;
    }
    else if constexpr (std::is_same_v<T, ListValue *>)
    {
        return static_cast<ListValue *>(m_value);
    }
    else if constexpr (std::is_pointer_v<T>)
    {
        return nullptr;
    }
    else
    {
        return T{}; // Return default-constructed value for non-pointer types
    }
}