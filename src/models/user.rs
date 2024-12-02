use std::{error::Error, io::Read};

use argon2::{password_hash::{self, rand_core::OsRng, SaltString}, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use serde_json::{json, Value};
use sqlx::{prelude::FromRow, query, query_as, PgPool};

#[derive(FromRow)]
pub struct User {
    login: String,
    password_hash: String,
    details: Value
}

impl User {
    /// ## User::select
    /// 
    /// Selects a user from the database
    /// 
    pub async fn query(
        conn: &PgPool,
        login: String
    ) -> Result<User, Box<dyn Error>> {
        let mut tx = match conn.begin().await {
            Ok(tx) => tx,
            Err(_) => return Err("Something went wrong.".into())
        };

        let sql = "
            SELECT 
                *
            FROM
                users
            WHERE
                login = $1
            ;
        ";
        let q = query_as(&sql)
            .bind(&login);

        let result = match q.fetch_one(&mut *tx).await {
            Ok(user) => user,
            Err(_) => return Err("User not found".into())
        };

        let _ = tx.commit().await;

        return Ok(result);
    }

    /// ## User::insert
    /// 
    /// Inserts a user with provided data into the database <br>
    /// 
    /// Errors:
    /// + when a user with provided login already exist
    /// + when the login is longer than 255 chars
    /// 
    pub async fn insert(
        conn: &PgPool,
        login: String,
        password: String,
        details: Value
    ) -> Result<(), Box<dyn Error>> {
        let mut tx = match conn.begin().await {
            Ok(tx) => tx,
            Err(_) => return Err("Something went wrong.".into())
        };

        let sql = "
            INSERT INTO
                users (login, password_hash, details)
            VALUES
                ($1, $2, $3)
            ;
        ";
        
        let pwd = password.as_bytes();
        let salt = SaltString::generate(&mut OsRng);

        let password_hash = match Argon2::default().hash_password(pwd, &salt) {
            Ok(hash) => hash,
            Err(_) => return Err("failed to create the password hash.".into())
        }
        .to_string();

        let q = query(sql)
            .bind(&login)
            .bind(password_hash)
            .bind(&details);

        match q.execute(&mut *tx).await {
            Ok(_) => (),
            Err(_) => return Err("This user already exists".into())
        };

        let _ = tx.commit().await;

        return Ok(());
    }


    /// ## User::delete
    /// 
    /// Deletes a user and all of it's related data from the database
    /// 
    pub async fn delete(
        conn: &PgPool,
        login: String
    ) -> Result<(), Box<dyn Error>> {
        let mut tx = match conn.begin().await {
            Ok(tx) => tx,
            Err(_) => return Err("Something went wrong.".into())
        };

        let sql = "DELETE FROM users WHERE login = $1";
        let q = query(sql)
            .bind(&login);
        let _ = q.execute(&mut *tx).await;

        let sql = "DELETE FROM users_groups WHERE user_login = $1";
        let q = query(sql)
            .bind(&login);
        let _ = q.execute(&mut *tx).await;

        let _ = tx.commit().await;

        return Ok(());
    }

    /// ## User::has_permission
    /// 
    /// Check if a user has a specified permission
    /// 
    /// Errors:
    /// + When the user do not have the provided permission
    /// 
    pub async fn has_permissions(
        self: &Self,
        conn: &PgPool,
        permission_name: String
    ) -> Result<(), Box<dyn Error>> {
        let mut tx = match conn.begin().await {
            Ok(tx) => tx,
            Err(_) => return Err("Something went wrong.".into())
        };

        let sql = "
            SELECT
                gp.permission_name
            FROM
                users u
            INNER JOIN
                users_groups ug
            ON
                u.login = ug.user_login
            INNER JOIN
                groups_permissions gp
            ON
                ug.group_name = gp.group_name
            WHERE
                u.login = '$1
            AND
                gp.permission_name  = $2
            LIMIT
                1;
        ";
        let q = query(sql)
            .bind(&self.login)
            .bind(&permission_name);
        let num_rows = q
            .execute(&mut *tx)
            .await
            .unwrap()
            .rows_affected();

        if num_rows < 1 {
            return Err("This user do not have this permission".into());
        }

        let _ = tx.commit().await;

        return Ok(());
    }
}
