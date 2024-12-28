#![allow(unused)]

use std::error::Error;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, query, query_as, PgConnection};
use crate::{models::{Order, event::{Event, EventType}}, util::string::json_value_to_pretty_string};

#[derive(FromRow, Deserialize, Serialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Permission {
    pub name: String,
    pub description: String,
}

impl ToString for Permission {
    fn to_string(&self) -> String {
        let formatted = json_value_to_pretty_string(&serde_json::to_value(&self).unwrap());

        return formatted;
    }
}

pub enum PermissionListError {}

impl ToString for PermissionListError {
    fn to_string(&self) -> String {
        return "".to_string();
    }
}

pub enum PermissionRetrieveError {
    /// Returned when a permission with specified name is not found
    NotFound,
}

impl ToString for PermissionRetrieveError {
    fn to_string(&self) -> String {
        return match self {
            Self::NotFound => "Permission with this name cannot be found".to_string()
        };
    }
}

pub enum PermissionInsertError {
    /// Returned when the permission either has too long name or description
    /// or when a permission with provided name already exist
    NameError
}

impl ToString for PermissionInsertError {
    fn to_string(&self) -> String {
        return match self {
            Self::NameError => "Either permission name or description is too long or permission with this name already exist.".to_string()
        }
    }
}

pub enum PermissionDeleteError {}

impl ToString for PermissionDeleteError {
    fn to_string(&self) -> String {
        return "".to_string();
    }
}

impl Permission {
    /// ## Permission::list
    /// 
    /// Lists number of permissions in specified order with specified offset from the database
    /// 
    pub async fn list(
        conn: &mut PgConnection,
        order: Option<Order>,
        offset: Option<usize>,
        limit: Option<usize>
    ) -> Result<Vec<Self>, PermissionListError> {
        let order = order.unwrap_or(Order::Ascending);
        let offset = offset.unwrap_or(0);
        let limit = limit.unwrap_or(10);

        let sql = format!(
            "SELECT * FROM permissions ORDER BY {} OFFSET {} ROWS LIMIT {};",
            order.to_string(),
            offset,
            limit
        );
        let result = query_as(&sql)
            .fetch_all(&mut *conn)
            .await
            .unwrap();

        return Ok(result);
    }

    /// ## Permission::retrieve
    /// 
    /// Retrieves a permission with specified name from the database
    /// 
    /// Errors:
    /// + when permission with specified name do not exist
    /// 
    pub async fn retrieve(
        conn: &mut PgConnection,
        name: &String
    ) -> Result<Self, PermissionRetrieveError> {
        let sql = "SELECT * FROM permissions WHERE name = $1;";
        let result = query_as(&sql)
            .bind(&name)
            .fetch_one(&mut *conn)
            .await;

        match result {
            Ok(result) => return Ok(result),
            Err(_) => return Err(PermissionRetrieveError::NotFound)
        };
    }

    /// ## Permission::insert
    /// 
    /// Inserts a permission with provided data into the database <br>
    /// 
    /// Errors:
    /// + when a permission with provided name already exist
    /// + when the name is longer than 255 chars or description is longer than 3000 chars
    /// 
    pub async fn insert(
        conn: &mut PgConnection,
        name: &String,
        description: &String
    ) -> Result<(), PermissionInsertError> {
        let sql = "INSERT INTO permissions (name, description) VALUES ($1, $2);".to_string();
        let q = query(&sql).bind(&name).bind(&description);

        match q.execute(&mut *conn).await {
            Ok(_) => return Ok(()),
            Err(_) => return Err(PermissionInsertError::NameError)
        };
    }

    /// ## Permission::delete
    /// 
    /// Deletes a permission with provided name from the database
    /// 
    pub async fn delete(
        conn: &mut PgConnection,
        name: &String
    ) -> Result<(), PermissionDeleteError> {
        let sql = "DELETE FROM permissions WHERE name = $1;";
        let q = query(&sql)
            .bind(&name)
            .execute(&mut *conn)
            .await;
        
        return Ok(());
    }


    /// ## Permission::event
    ///
    /// Get an PermissionEvent instance for permission event creation
    ///
    pub fn event() -> PermissionEvent {
        return PermissionEvent;
    }
}


struct PermissionEvent;


impl PermissionEvent {
    /// ## PermissionEvent::insert
    ///
    /// Insert a PermissionCreate event into database
    ///
    pub async fn insert(
        conn: &mut PgConnection,
        name: String,
        description: String
    ) {
        let data = Permission {
            name,
            description
        };
        let data = serde_json::to_value(&data).unwrap();

        let _ = Event::insert(conn, EventType::PermissionCreate, data).await;
    }


    /// ## PermissionEvent::delete
    ///
    /// Insert a PermissionDelete event into database
    ///
    pub async fn delete(
        conn: &mut PgConnection,
        name: String
    ) {
        let data = serde_json::to_value(&name).unwrap();

        let _ = Event::insert(conn, EventType::PermissionDelete, data).await;
    }
}
