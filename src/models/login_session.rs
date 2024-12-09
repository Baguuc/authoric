use std::{error::Error, os::linux::raw::stat};

use serde::{Deserialize, Serialize};
use sqlx::{query, query_as, FromRow, PgPool};

#[derive(Deserialize, Serialize, Clone, PartialEq, Eq)]
pub enum LoginSessionStatus {
    Commited,
    OnHold
}

impl From<String> for LoginSessionStatus {
    fn from(value: String) -> Self {
        return match value.as_str() {
           "Commited" => Self::Commited,
           "OnHold" => Self::OnHold,
            _ => todo!()
        };
    }
}

impl ToString for LoginSessionStatus {
    fn to_string(self: &Self) -> String {
        return match self {
            Self::Commited => "Commited",
            Self::OnHold => "OnHolds"
        }
        .to_string();
    }
}

#[derive(FromRow, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct LoginSessionRaw {
    pub id: i64,
    pub user_login: String,
    pub status: String
}

#[derive(Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct LoginSession {
    pub id: i64,
    pub user_login: String,
    pub status: LoginSessionStatus
}

pub struct LoginSessionUpdateData {
    pub status: LoginSessionStatus,
}

impl LoginSession {
    /// ## LoginSession::select
    /// 
    /// Selects a user's loggin session with specified id from the database
    /// 
    pub async fn select(
        conn: &PgPool,
        id: i64
    ) -> Result<Self, Box<dyn Error>> {
        let mut tx = match conn.begin().await {
            Ok(tx) => tx,
            Err(_) => return Err("Something went wrong.".into())
        };

        let sql = "
            SELECT 
                *
            FROM
                login_sessions
            WHERE
                id = $1
            ;
        ";
        let q = query_as(&sql)
            .bind(&id);

        let raw: LoginSessionRaw = match q.fetch_one(&mut *tx).await {
            Ok(raw) => raw,
            Err(_) => return Err("User not found".into())
        };

        let session = LoginSession {
            id: raw.id,
            user_login: raw.user_login,
            status: LoginSessionStatus::from(raw.status)
        };

        let _ = tx.commit().await;

        return Ok(session);
    }

    /// ## LoginSession::insert
    /// 
    /// Inserts a new login session with provided data into the database <br>
    /// 
    /// Errors:
    /// + when referenced user do not exist.
    /// 
    pub async fn insert(
        conn: &PgPool,
        user_login: String,
        status: LoginSessionStatus
    ) -> Result<i64, Box<dyn Error>> {
        let mut tx = match conn.begin().await {
            Ok(tx) => tx,
            Err(_) => return Err("Something went wrong.".into())
        };

        let sql = "
            INSERT INTO
                login_sessions (user_login, status)
            VALUES
                ($1, $2)
            RETURNING id;
            ;
        ";

        let q = query_as(sql)
            .bind(&user_login)
            .bind(status.to_string());

        let row: (i64,) = match q.fetch_one(&mut *tx).await {
            Ok(row) => row,
            Err(_) => return Err("This user do not exist".into())
        };
        let session_id = row.0;

        let _ = tx.commit().await;

        return Ok(session_id);
    }


    /// ## LoginSession::delete
    /// 
    /// Deletes a user's login session from the database (e.g. logs out the user)
    /// 
    pub async fn delete(
        conn: &PgPool,
        session_id: i64
    ) -> Result<(), Box<dyn Error>> {
        let mut tx = match conn.begin().await {
            Ok(tx) => tx,
            Err(_) => return Err("Something went wrong.".into())
        };

        let sql = "DELETE FROM login_sessions WHERE id = $1";
        let q = query(sql)
            .bind(&session_id);
        let _ = q.execute(&mut *tx).await;

        let _ = tx.commit().await;

        return Ok(());
    }
    
    /// ## LoginSession::update
    /// 
    /// Updates a login session with the specified new data
    /// 
    pub async fn update(
        conn: &PgPool,
        session_id: &i64,
        new_data: LoginSessionUpdateData
    ) -> Result<(), Box<dyn Error>> {
        let sql = "UPDATE login_sessions SET status = $1 WHERE id = $2;";
        let result = query(sql)
            .bind(new_data.status.to_string())
            .bind(session_id)
            .execute(conn)
            .await;
        
        return Ok(());
    }
}