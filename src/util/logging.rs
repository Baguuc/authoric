use colored::Colorize;
use serde_json::json;

use crate::util::string::json_value_to_pretty_string;

pub enum DatabaseOperationLogStatus<E>
where
    E: ToString,
{
    Ok,
    Err(E),
}

pub fn log_database_interaction<E>(
    title: &str,
    data: serde_json::Value,
    status: DatabaseOperationLogStatus<E>,
) where
    E: ToString,
{
    let status_string = match status {
        DatabaseOperationLogStatus::Ok => format!("{}", "OK".bright_green()),
        DatabaseOperationLogStatus::Err(details) => format!(
            "{}.\nDetails:\n{}",
            "FAILED".bright_red(),
            details.to_string()
        ),
    };

    log::debug!(
        "\n\n------===# {} #===------\n\nData:\n{}\n{}\n\n------===# {} #===------\n",
        title,
        json_value_to_pretty_string(&json!(data)),
        status_string,
        "*".repeat(title.len())
    );
}
