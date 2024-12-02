#![allow(unused)]

use std::error::Error;
use sqlx::{FromRow, query, query_as, PgPool};
use crate::models::Order;

#[derive(FromRow)]
pub struct Permission {
    name: String,
    description: String,
}

impl Permission {
    /// ## Permission::select
    /// 
    /// Selects all permissions in specified order or one permission with specified name
    /// 
    pub async fn select(
        conn: &PgPool,
        limit: Option<usize>,
        order_in: Option<Order>,
        with_name: Option<String>
    ) -> Result<Vec<Permission>, Box<dyn Error>> {
        let mut tx = match conn.begin().await {
            Ok(tx) => tx,
            Err(err) => return Err("Something went wrong.".into())
        };

        let sql = format!(
            "SELECT 
                *
            FROM 
                permissions 
            {} 
            {} 
            {};
            ", 
            // add where clause if needed
            match &with_name {
                Some(_) => "WHERE name = $1 ".to_string(),
                None => "".to_string()
            }, 
            // add order clause if needed
            match &order_in {
                Some(order_in) => format!("ORDER BY name {}", order_in.to_string()),
                None => "ORDER BY NAME ASC".to_string()
            },
            // add order limit if needed
            match &limit {
                Some(limit) => format!("LIMIT {}", limit),
                None => "".to_string()
            }
        );

        let mut q = query_as(&sql);

        if let Some(with_name) = &with_name {
            q = q.bind(with_name);
        }

        // this will never be an error even if the table is empty so unwrap is ok
        let result = q.fetch_all(&mut *tx).await.unwrap();

        let _ = tx.commit().await;

        return Ok(result);
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
        name: String,
        description: String
    ) -> Result<(), Box<dyn Error>> {
        let mut tx = match conn.begin().await {
            Ok(tx) => tx,
            Err(err) => return Err("Something went wrong.".into())
        };

        let sql = "INSERT INTO permissions (name, description) VALUES ($1, $2);".to_string();
        let q = query(&sql).bind(&name).bind(&description);

        match q.execute(&mut *tx).await {
            Ok(_) => (),
            Err(_) => return Err("Permission with this name already exist.".into())
        };

        let _ = tx.commit().await;

        return Ok(());
    }

    /// ## Permission::delete
    /// 
    /// Deletes a permission with provided name from the database
    /// 
    pub async fn delete(
        conn: &PgPool,
        name: String
    ) -> Result<(), Box<dyn Error>> {
        let mut tx = match conn.begin().await {
            Ok(tx) => tx,
            Err(err) => return Err("Something went wrong.".into())
        };

        let sql = "DELETE FROM permissions WHERE name = $1;".to_string();
        let q = query(&sql).bind(&name);

        q.execute(&mut *tx).await;
        let _ = tx.commit().await;

        return Ok(());
    }
}