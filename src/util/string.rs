use colored::Colorize;

pub fn json_value_to_pretty_string(value: &serde_json::Value) -> String {
    return json_value_to_pretty_string_with_prefix(value, 0);
}

fn json_value_to_pretty_string_with_prefix(value: &serde_json::Value, indent_times: usize) -> String {
    let indent_prefix = "  ".repeat(indent_times);
    let attribute_prefix = "+ "
        .green()
        .to_string();
    let list_item_prefix = "- "
        .green()
        .to_string();

    if value.is_boolean() {
        return value
            .as_bool()
            .unwrap()
            .to_string()
            .green()
            .to_string();
    }

    if value.is_f64() {
        return value
            .as_f64()
            .unwrap()
            .to_string()
            .green()
            .to_string();
    }

    if value.is_i64() {
        return value
            .as_i64()
            .unwrap()
            .to_string()
            .green()
            .to_string();
    }

    if value.is_null() {
        return "NULL"
            .to_string()
            .green()
            .to_string();
    }

    if value.is_number() {
        return value
            .as_number()
            .unwrap()
            .to_string()
            .green()
            .to_string();
    }

    if value.is_string() {
        return value
            .as_str()
            .unwrap()
            .to_string()
            .green()
            .to_string();
    }

    if value.is_u64() {
        return value
            .as_u64()
            .unwrap()
            .to_string()
            .green()
            .to_string();
    }

    if value.is_array() {
        let array = value
            .as_array()
            .unwrap(); 
        let mut formatted = String::new();
        let mut i = 0;
        
        for item in array {
            i += 1;

            if item.is_array() || item.is_object() {
                formatted += format!(
                    "{}{}{}:\n{}",
                    indent_prefix,
                    list_item_prefix,
                    i,
                    json_value_to_pretty_string_with_prefix(item, indent_times+1)
                )
                .as_str();
            } else {
                formatted += format!(
                    "{}{}{}: {}\n",
                    indent_prefix,
                    list_item_prefix,
                    i,
                    json_value_to_pretty_string_with_prefix(item, 0)
                )
                .as_str();
            }
        }

        return formatted;
    }

    if value.is_object() {
        let obj = value.as_object().unwrap();
        let mut formatted = String::new();

        for (key, value) in obj.iter() {
            if value.is_array() || value.is_object() {
                formatted += format!(
                    "{}{}{}:\n{}",
                    indent_prefix,
                    attribute_prefix,
                    key,
                    json_value_to_pretty_string_with_prefix(value, indent_times+1)
                )
                .as_str();
            } else {
                formatted += format!(
                    "{}{}{}: {}\n",
                    indent_prefix,
                    attribute_prefix,
                    key,
                    json_value_to_pretty_string_with_prefix(value, 0)
                )
                .as_str();
            }
        }

        return formatted;
    }

    return String::new();
}