#![allow(unused)]

use std::error::Error;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as, FromRow, PgConnection};
use crate::{
    models::{
        Order,
        Permission
    },
    util::string::json_value_to_pretty_string
};

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

#[derive(Debug)]
pub enum GroupListError {}

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
  /// Returned when the group with specified name already exist
  NameError,
  /// Returned when one of the permissions listed do not exist in the database
  PermissionNotFound
}

impl ToString for GroupInsertError {
  fn to_string(&self) -> String {
    return match self {
        Self::NameError => "A group with provided name do not exist.",
        Self::PermissionNotFound => "One of the listed permissions to not exists"
    }
    .to_string();
  }
}

pub enum GroupDeleteError {
    /// Returned when the group with specified name do not exist
    NotFound
}

impl ToString for GroupDeleteError {
  fn to_string(&self) -> String {
    return match self {
        Self::NotFound => "A group with this name do not exist."
    }
    .to_string();
  }
}

pub enum GroupGrantError {
  /// Returned when a group with provided name do not exist
  NotFound,
  /// Returned when permission with provided name do not exist
  PermissionNotFound
}

impl ToString for GroupGrantError {
  fn to_string(&self) -> String {
    return match self {
      Self::NotFound => "A group with provided name do not exist",
      Self::PermissionNotFound => "A permission with provided name do not exist"
    }
    .to_string();
  }
}

pub enum GroupRevokeError {
  /// Returned when a group with provided name do not exist
  NotFound,
  /// Returned when permission with provided name do not exist
  PermissionNotFound,
  /// Returned when the provided permission wasn't granted
  PermissionNotGranted
}

impl ToString for GroupRevokeError {
  fn to_string(&self) -> String {
    return match self {
      Self::NotFound => "A group with provided name do not exist",
      Self::PermissionNotFound => "A permission with provided name do not exist",
      Self::PermissionNotGranted => "The group with provided name never had this permission"
    }
    .to_string();
  }
}

impl Group {
   /// ## Group::list
  /// 
  /// Lists number of groups in specified order with specified offset from the database
  /// 
  pub async fn list(
    conn: &mut PgConnection,
    order: Option<Order>,
    offset: Option<usize>,
    limit: Option<usize>
  ) -> Result<Vec<Self>, GroupListError> {
    let order = order.unwrap_or(Order::Ascending);
    let offset = offset.unwrap_or(0);
    let limit = limit.unwrap_or(10);

    let sql = format!("
    SELECT 
      g.name,
      g.description,
      ARRAY_REMOVE(ARRAY_AGG(gp.permission_name), NULL) AS permissions
    FROM groups g
    INNER JOIN groups_permissions gp ON gp.group_name = g.name
    GROUP BY g.name
    ORDER BY g.name {}
    OFFSET {} ROWS
    limit {};
    ", order.to_string(), offset, limit);

    let result = query_as(&sql)
      .fetch_all(&mut *conn)
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
    conn: &mut PgConnection,
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
      .fetch_one(&mut *conn)
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
    conn: &mut PgConnection,
    name: &String,
    description: &String,
    permissions: &Vec<String>
  ) -> Result<(), GroupInsertError> {
    let sql = "INSERT INTO groups (name, description) VALUES ($1, $2);".to_string();
    let q = query(&sql).bind(&name).bind(&description);

    match q.execute(&mut *conn).await {
      Ok(_) => (),
      Err(_) => return Err(GroupInsertError::NameError)
    };

    for permission_name in permissions {
      match Self::grant_permission(&mut *conn, &name, permission_name).await {
        Ok(_) => (),
        Err(_) => return Err(GroupInsertError::PermissionNotFound)
      }
    }

    return Ok(());
  }

  
  /// ## Group::delete
  /// 
  /// Deletes a group and all of it's related data from the database
  /// 
  pub async fn delete(
    conn: &mut PgConnection,
    name: &String
  ) -> Result<(), GroupDeleteError> {
    let sql = "DELETE FROM groups_permissions WHERE group_name = $1;";
    let _ = query(&sql)
        .bind(&name)
        .execute(&mut *conn)
        .await;

    let sql = "DELETE FROM groups WHERE name = $1;".to_string();
    let result = query(&sql)
        .bind(&name)
        .execute(&mut *conn)
        .await
        .unwrap();

    if result.rows_affected() == 0 {
        return Err(GroupDeleteError::NotFound);
    }
    
    return Ok(());
  }

  /// ## Group::has_permission
  /// 
  /// Checks if group has a specified permission
  /// 
  pub async fn has_permission(
    conn: &mut PgConnection,
    name: &String,
    permission_name: &String
  ) -> Result<bool, GroupRetrieveError> {
    let data = Self::retrieve(conn, &name).await?;

    return Ok(data.permissions.contains(&permission_name));
  }

  /// ## Group::grant_permission
  /// 
  /// Grants group a permission with specified name
  /// 
  /// Errors:
  /// + When provided group do not exist
  /// + When the provided permission do not exist
  /// 
  pub async fn grant_permission(
    conn: &mut PgConnection,
    name: &String,
    permission_name: &String
  ) -> Result<(), GroupGrantError> {
    if let Err(_) = Permission::retrieve(conn, permission_name).await {
        return Err(GroupGrantError::PermissionNotFound);
    }

    if let Err(_) = Group::retrieve(conn, name).await {
        return Err(GroupGrantError::NotFound);
    }

    let sql = "INSERT INTO groups_permissions (group_name, permission_name) VALUES ($1, $2);";
    let result = query(sql)
      .bind(name)
      .bind(permission_name)
      .execute(&mut *conn)
      .await;

    return Ok(());
  }

  /// ## Group::revoke_permission
  /// 
  /// Revokes a permission from group with specified name
  /// 
  /// Errors:
  /// + When provided group or permission do not exist
  /// 
  pub async fn revoke_permission(
    conn: &mut PgConnection,
    name: &String,
    permission_name: &String
  ) -> Result<(), GroupRevokeError> {
    if let Err(_) = Permission::retrieve(conn, permission_name).await {
        return Err(GroupRevokeError::PermissionNotFound);
    }

    if let Err(_) = Group::retrieve(conn, name).await {
        return Err(GroupRevokeError::NotFound);
    }

    let sql = "DELETE FROM groups_permissions WHERE group_name = $1 AND permission_name = $2;";
    let result = query(sql)
      .bind(name)
      .bind(permission_name)
      .execute(&mut *conn)
      .await
      .unwrap();

    if result.rows_affected() == 0 {
        return Err(GroupRevokeError::PermissionNotGranted)
    }

    return Ok(());
  }
}
