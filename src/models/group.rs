#![allow(unused)]

use std::error::Error;
use sqlx::{FromRow, query, query_as, PgPool};
use crate::models::Order;

use super::permission::Permission;


#[derive(FromRow)]
pub struct Group {
    name: String,
    description: String,
    permissions: Vec<String>
}

impl Group {
    /// ## Group::select
    /// 
    /// Selects all groups in specified order or one groups with specified name
    /// 
    pub async fn select(
        conn: &PgPool,
        limit: Option<usize>,
        order_in: Option<Order>,
        with_name: Option<String>
    ) -> Result<Vec<Self>, Box<dyn Error>> {
        let mut tx = match conn.begin().await {
            Ok(tx) => tx,
            Err(err) => return Err("Something went wrong.".into())
        };

        let sql = format!("
            SELECT 
                g.name, 
                g.description, 
                ARRAY_REMOVE(ARRAY_AGG(p.name), NULL) permissions 
            FROM 
                groups g 
            INNER JOIN 
                groups_permissions gp 
            ON 
                g.name = gp.group_name 
            INNER JOIN 
                permissions p 
            ON 
                gp.permission_name = p.name 
            {} 
            GROUP BY 
                g.name 
            {} 
            {};
            ", 
            // add where clause if needed
            match &with_name {
                Some(_) => "WHERE name = $1".to_string(),
                None => "".to_string()
            }, 
            // add order clause if needed
            match &order_in {
                Some(order_in) => format!("ORDER BY g.name {}", order_in.to_string()),
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
    ) -> Result<(), Box<dyn Error>> {
        let mut tx = match conn.begin().await {
            Ok(tx) => tx,
            Err(err) => return Err("Something went wrong.".into())
        };

        let sql = "INSERT INTO groups (name, description) VALUES ($1, $2);".to_string();
        let q = query(&sql).bind(&name).bind(&description);

        match q.execute(&mut *tx).await {
            Ok(_) => (),
            Err(_) => return Err("Group with this name already exist.".into())
        };

        for permission in &permissions {
            match query("INSERT INTO groups_permissions (group_name, permission_name) VALUES ($1, $2);").bind(&name).bind(&permission).execute(&mut *tx).await {
                Ok(_) => (),
                Err(_) => return Err("Cannot assign non-existent permissions to a group.".into())
            }
        }

        let _ = tx.commit().await;

        return Ok(());
    }

    
    /// ## Group::delete
    /// 
    /// Deletes a group and all of it's related data from the database
    /// 
    pub async fn delete(
        conn: &PgPool,
        name: String
    ) -> Result<(), Box<dyn Error>> {
        let mut tx = match conn.begin().await {
            Ok(tx) => tx,
            Err(err) => return Err("Something went wrong.".into())
        };
        
        let sql = "DELETE FROM groups_permissions WHERE group_name = $1;";
        let q = query(&sql).bind(&name);

        match q.execute(&mut *tx).await {
            Ok(_) => (),
            Err(_) => return Err("There was a problem deleting the group's permission relations.".into())
        };

        let sql = "DELETE FROM groups WHERE name = $1;".to_string();
        let q = query(&sql).bind(&name);

        match q.execute(&mut *tx).await {
            Ok(_) => (),
            Err(_) => return Err("This group does not exist.".into())
        };

        let _ = tx.commit().await;

        return Ok(());
    }
}
