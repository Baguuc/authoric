#![allow(unused)]

use std::error::Error;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, query, query_as, PgPool};
use crate::{models::Order, util::string::json_value_to_pretty_string};

use super::permission::Permission;


#[derive(FromRow, Deserialize, Serialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Group {
    pub name: String,
    pub description: String,
    pub permissions: Vec<String>
}

impl ToString for Group {
    fn to_string(&self) -> String {
        let formatted = json_value_to_pretty_string(&serde_json::to_value(&self).unwrap());

        return formatted;
    }
}

pub enum GroupListError {}

impl ToString for GroupListError {
    fn to_string(&self) -> String {
        return "".to_string();
    }
}

pub enum GroupRetrieveError {
    /// Returned when a group with specified name is not found
    NotFound,
}

impl ToString for GroupRetrieveError {
    fn to_string(&self) -> String {
        return match self {
            Self::NotFound => "Group with this name cannot be found".to_string()
        };
    }
}

pub enum GroupInsertError {
    /// Returned when the group either has too long name or description,
    /// a group with provided name already exist
    /// or one of the provided permissions do not exist
    NameError,
    /// Returned when transaction fails for some reason,
    /// also contains the original error string
    ServerError(String)
}

impl ToString for GroupInsertError {
    fn to_string(&self) -> String {
        return match self {
            Self::NameError => "Either the provided name or description is too long, this group already exist or one of the provided permissions do not exist.".to_string(),
            Self::ServerError(original_err) => format!("Server error: {}", original_err)
        };
    }
}

pub enum GroupDeleteError {
    /// Returned when transaction fails for some reason,
    /// also contains the original error string
    ServerError(String)
}

impl ToString for GroupDeleteError {
    fn to_string(&self) -> String {
        return match self {
            Self::ServerError(original_err) => format!("Server error: {}", original_err)
        };
    }
}

impl Group {
     /// ## Group::list
    /// 
    /// Lists number of groups in specified order with specified offset from the database
    /// 
    pub async fn list(
        conn: &PgPool,
        order: Option<Order>,
        offset: Option<usize>,
        limit: Option<usize>
    ) -> Result<Vec<Self>, GroupListError> {
        let order = order.unwrap_or(Order::Ascending);
        let offset = offset.unwrap_or(0);
        let limit = limit.unwrap_or(10);

        let sql = format!(
            "SELECT * FROM groups ORDER BY {} OFFSET {} ROWS LIMIT {};",
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

    /// ## Group::retrieve
    /// 
    /// Retrieves a group with specified name from the database
    /// 
    /// Errors:
    /// + when group with specified name do not exist
    /// 
    pub async fn retrieve(
        conn: &PgPool,
        name: &String
    ) -> Result<Self, GroupRetrieveError> {
        let sql = "
        SELECT 
            g.name,
            g.description,
            ARRAY_REMOVE(ARRAY_AGG(gp.permission_name), NULL) AS permissions
        FROM 
            groups g
        INNER JOIN
            groups_permissions gp
        ON
            gp.group_name = g.name
        WHERE
            name = $1
        GROUP BY
            g.name;
        ";
        let result = query_as(&sql)
            .bind(&name)
            .fetch_one(conn)
            .await;

        match result {
            Ok(result) => return Ok(result),
            Err(_) => return Err(GroupRetrieveError::NotFound)
        };
    }

    /// ## Group::insert
    /// 
    /// Inserts a group with provided data into database <br>
    /// 
    /// Errors:
    /// + when a group with provided name already exist
    /// + when the name is longer than 255 chars or description is longer than 3000 chars
    /// + when the at least one of assigned permissions do not exist
    /// 
    pub async fn insert(
        conn: &PgPool,
        name: String,
        description: String,
        permissions: Vec<String>
    ) -> Result<(), GroupInsertError> {
        let mut tx = match conn.begin().await {
            Ok(tx) => tx,
            Err(err) => return Err(GroupInsertError::ServerError(err.to_string()))
        };

        let sql = "INSERT INTO groups (name, description) VALUES ($1, $2);".to_string();
        let q = query(&sql).bind(&name).bind(&description);

        match q.execute(&mut *tx).await {
            Ok(_) => (),
            Err(_) => return Err(GroupInsertError::NameError)
        };

        for permission in &permissions {
            match query("INSERT INTO groups_permissions (group_name, permission_name) VALUES ($1, $2);").bind(&name).bind(&permission).execute(&mut *tx).await {
                Ok(_) => (),
                Err(_) => return Err(GroupInsertError::NameError)
            }
        }

        let _ = match tx.commit().await {
            Ok(_) => (),
            Err(err) => return Err(GroupInsertError::ServerError(err.to_string()))
        };

        return Ok(());
    }

    
    /// ## Group::delete
    /// 
    /// Deletes a group and all of it's related data from the database
    /// 
    pub async fn delete(
        conn: &PgPool,
        name: String
    ) -> Result<(), GroupDeleteError> {
        let mut tx = match conn.begin().await {
            Ok(tx) => tx,
            Err(err) => return Err(GroupDeleteError::ServerError(err.to_string()))
        };
        
        let sql = "DELETE FROM groups_permissions WHERE group_name = $1;";
        let q = query(&sql).bind(&name);

        let _ = q.execute(&mut *tx).await;

        let sql = "DELETE FROM groups WHERE name = $1;".to_string();
        let q = query(&sql).bind(&name);

        match q.execute(&mut *tx).await {
            Ok(_) => (),
            Err(err) => return Err(GroupDeleteError::ServerError(err.to_string()))
        };

        let _ = tx.commit().await;

        return Ok(());
    }
}
