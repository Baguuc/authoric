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
    log::debug!("---");
    log::debug!(
        "{}\nData:\n{}",
        title,
        json_value_to_pretty_string(&json!(data))
    );
    match status {
        DatabaseOperationLogStatus::Ok => log::debug!("{}", "OK".bright_green()),
        DatabaseOperationLogStatus::Err(details) => log::debug!(
            "{}.\nDetails:\n{}",
            "FAILED".bright_red(),
            details.to_string()
        ),
    };
    log::debug!("---");
}
