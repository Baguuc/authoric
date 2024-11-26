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
    /// ## Permission::retrieve
    /// 
    /// Creates a RetrieveQuery instance
    /// 
    pub fn retrieve() -> RetrieveQuery {
        return RetrieveQuery::new();
    }

    /// ## Permission::retrieve
    /// 
    /// Creates a CreateQuery instance
    /// 
    pub fn create(name: String, description: String) -> CreateQuery {
        return CreateQuery::new(name, description);
    }

    /// ## Permission::retrieve
    /// 
    /// Creates a RetrieveQuery instance
    /// 
    pub fn delete(name: String) -> DeleteQuery {
        return DeleteQuery::new(name);
    }

    /// ## Permission::new
    /// 
    /// Creates a instance of Permission
    /// 
    fn new(name: String, description: String) -> Self {
        return Self {
            name,
            description
        }
    }
}

pub struct RetrieveQuery {
    limit: Option<usize>,
    order_in: Option<Order>,
    with_name: Option<String>
}

impl RetrieveQuery {
    /// ## RetrieveQuery::with_limit
    /// 
    /// Sets the max limit to returned rows <br>
    /// default: 10
    /// 
    pub fn with_limit(&mut self, limit: usize) -> &mut Self {
        self.limit = Some(limit);

        return self;
    }

    /// ## RetrieveQuery::in_order
    /// 
    /// Sets the order of the returned result <br>
    /// default: ASC (ascending)
    /// 
    pub fn in_order(&mut self, order_in: Order) -> &mut Self {
        self.order_in = Some(order_in);

        return self;
    }

    /// ## RetrieveQuery::with_name
    /// 
    /// Sets the name to filter the permissions by <br>
    /// default: None
    /// 
    pub fn with_name(&mut self, with_name: String) -> &mut Self {
        self.with_name = Some(with_name);

        return self;
    }

    /// ## RetrieveQuery::query
    /// 
    /// Queries the data matching provided criteria from the database
    /// 
    pub async fn query(&self, conn: &PgPool) -> Result<Vec<Permission>, Box<dyn Error>> {
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
            match &self.with_name {
                Some(_) => "WHERE name = $1 ".to_string(),
                None => "".to_string()
            }, 
            // add order clause if needed
            match &self.order_in {
                Some(order_in) => format!("ORDER BY name {}", order_in.to_string()),
                None => "ORDER BY NAME ASC".to_string()
            },
            // add order limit if needed
            match &self.limit {
                Some(limit) => format!("LIMIT {}", limit),
                None => "".to_string()
            }
        );

        let mut q = query_as(&sql);

        if let Some(with_name) = &self.with_name {
            q = q.bind(with_name);
        }

        // this will never be an error even if the table is empty so unwrap is ok
        let result = q.fetch_all(&mut *tx).await.unwrap();

        let _ = tx.commit().await;

        return Ok(result);
    }

    /// ## RetrieveQuery::new
    /// 
    /// Creates a new instance of RetrieveQuery
    /// 
    fn new() -> Self {
        return Self {
            limit: Some(10),
            order_in: Some(Order::Ascending),
            with_name: None
        };
    }
}

pub struct CreateQuery {
    name: String,
    description: String
}

impl CreateQuery {
    /// ## CreateQuery::query
    /// 
    /// Creates a permission with provided data <br>
    /// 
    /// Errors:
    /// + when a permission with provided name already exist
    /// + when the name is longer than 255 chars or description is longer than 3000 chars
    /// 
    pub async fn query(&self, conn: &PgPool) -> Result<(), Box<dyn Error>> {
        let mut tx = match conn.begin().await {
            Ok(tx) => tx,
            Err(err) => return Err("Something went wrong.".into())
        };

        let sql = "INSERT INTO permissions (name, description) VALUES ($1, $2);".to_string();
        let q = query(&sql).bind(&self.name).bind(&self.description);

        match q.execute(&mut *tx).await {
            Ok(_) => (),
            Err(_) => return Err("Permission with this name already exist.".into())
        };

        let _ = tx.commit().await;

        return Ok(());
    }

    /// ## CreateQuery::new
    /// 
    /// Creates a new instance of CreateQuery
    /// 
    fn new(name: String, description: String) -> Self {
        return Self {
            name,
            description
        };
    }
}

pub struct DeleteQuery {
    name: String,
}

impl DeleteQuery {
    /// ## CreateQuery::query
    /// 
    /// Creates a permission with provided data
    /// 
    /// Errors:
    /// + Returns an error when a permission with provided name does not exist
    /// 
    pub async fn query(&self, conn: &PgPool) -> Result<(), Box<dyn Error>> {
        let mut tx = match conn.begin().await {
            Ok(tx) => tx,
            Err(err) => return Err("Something went wrong.".into())
        };

        let sql = "DELETE FROM permissions WHERE name = $1;".to_string();
        let q = query(&sql).bind(&self.name);

        match q.execute(&mut *tx).await {
            Ok(_) => (),
            Err(_) => return Err("This permission does not exist.".into())
        };

        let _ = tx.commit().await;

        return Ok(());
    }

    /// ## DeleteQuery::new
    /// 
    /// Creates a new instance of DeleteQuery
    /// 
    pub fn new(name: String) -> Self {
        return Self {
            name
        };
    }
}