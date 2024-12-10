#![allow(unused)]

use std::error::Error;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, query, query_as, PgPool};
use crate::models::Order;

#[derive(FromRow, Deserialize, Serialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Permission {
    pub name: String,
    pub description: String,
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
        conn: &PgPool,
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
            .fetch_all(conn)
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
        conn: &PgPool,
        name: &String
    ) -> Result<Self, PermissionRetrieveError> {
        let sql = "SELECT * FROM permissions WHERE name = $1;";
        let result = query_as(&sql)
            .fetch_one(conn)
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
        conn: &PgPool,
        name: &String,
        description: &String
    ) -> Result<(), PermissionInsertError> {
        let sql = "INSERT INTO permissions (name, description) VALUES ($1, $2);".to_string();
        let q = query(&sql).bind(&name).bind(&description);

        match q.execute(conn).await {
            Ok(_) => return Ok(()),
            Err(_) => return Err(PermissionInsertError::NameError)
        };
    }

    /// ## Permission::delete
    /// 
    /// Deletes a permission with provided name from the database
    /// 
    pub async fn delete(
        conn: &PgPool,
        name: &String
    ) -> Result<(), PermissionDeleteError> {
        let sql = "DELETE FROM permissions WHERE name = $1;";
        let q = query(&sql)
            .bind(&name)
            .execute(conn)
            .await;
        
        return Ok(());
    }
}