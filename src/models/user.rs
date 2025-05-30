use crate::{
    models::{
        login_session::{
            LoginSession, LoginSessionDeleteError, LoginSessionInsertError,
            LoginSessionRetrieveError,
        },
        Order,
    },
    util::logging::{log_database_interaction, DatabaseOperationLogStatus},
};
use crate::{util::string::json_value_to_pretty_string, web::ServerResponse};
use actix_web::http::StatusCode;
use argon2::{
    password_hash::{self, rand_core::OsRng, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{prelude::FromRow, query, query_as, PgConnection};

use super::Group;

#[derive(FromRow, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct User {
    pub login: String,
    pub password_hash: String,
    pub details: Value,
}

impl ToString for User {
    fn to_string(&self) -> String {
        let formatted = json_value_to_pretty_string(&serde_json::to_value(&self).unwrap());

        return formatted;
    }
}

#[derive(FromRow, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct UserCredentials {
    pub login: String,
    pub password: String,
}

pub type UserListError = ();

#[derive(Debug)]
pub enum UserRetrieveError {
    /// Returned when a user with specified login is not found
    NotFound,
}

impl ToString for UserRetrieveError {
    fn to_string(&self) -> String {
        return match self {
            Self::NotFound => "This user cannot be found".to_string(),
        };
    }
}

#[derive(Debug)]
pub enum UserInsertError {
    /// Returned when the user either has too long login,
    /// a user with provided login already exist
    /// or one of the provided groups do not exist
    NameError,
    /// Returned when the provided password cannot be hashed
    CannotHash(String),
}

impl ToString for UserInsertError {
    fn to_string(&self) -> String {
        return match self {
      Self::NameError => "Either the provided login is too long, this user already exist or one of the provided groups do not exist.".to_string(),
      Self::CannotHash(err) => format!("Password hashing error: {}.", err)
    };
    }
}

pub enum UserDeleteError {
    /// Returned when the user with specified login do not exist
    NotFound,
}

pub enum UserHasPermissionError {
    /// Returned when the user do not have queried permissions
    Unauthorized,
}

impl ToString for UserHasPermissionError {
    fn to_string(&self) -> String {
        return match self {
            Self::Unauthorized => "This user do not have this permission".to_string(),
        };
    }
}

#[derive(Debug)]
pub enum UserLoginError {
    /// Returned when the user is not found
    NotFound,
    /// Returned when the credentials are invalid
    InvalidCredentials,
    /// Returned when the token hash cannot be created
    CannotHash(String),
}

pub enum UserGrantError {
    /// Returned when the user with specified login do not exist
    NotFound,
    /// Returned when the group with specified name do not exist
    GroupNotFound,
}

impl ToString for UserGrantError {
    fn to_string(&self) -> String {
        return match self {
            Self::NotFound => "Provided user do not exist".to_string(),
            Self::GroupNotFound => "Provided group do not exist".to_string(),
        };
    }
}

pub enum UserRevokeError {
    /// Returned when the user with specified login do not exist
    NotFound,
    /// Returned when the group with specified name do not exist
    GroupNotFound,
    /// Returned when the group didn't had specified permission granted
    NotGranted,
}

impl ToString for UserRevokeError {
    fn to_string(&self) -> String {
        return match self {
            Self::NotFound => "Provided user do not exist".to_string(),
            Self::GroupNotFound => "Provided group do not exist".to_string(),
            Self::NotGranted => "Provided group do not had this permission".to_string(),
        };
    }
}

pub enum UserVerifyPasswordError {
    /// Returned when the user is not found
    NotFound,
    /// Returned when the credentials are invalid
    Unauthorized,
}

impl User {
    /// ## User::list
    ///
    /// Lists number of users in specified order with specified offset from the database
    ///
    pub async fn list(
        conn: &mut PgConnection,
        order: Option<Order>,
        offset: Option<usize>,
        limit: Option<usize>,
    ) -> Result<Vec<Self>, UserListError> {
        let order = order.unwrap_or(Order::Ascending);
        let offset = offset.unwrap_or(0);
        let limit = limit.unwrap_or(10);
        let sql = format!(
            "SELECT * FROM users ORDER BY login {} OFFSET {} ROWS LIMIT {};",
            order.to_string(),
            offset,
            limit
        );
        let result = query_as(&sql).fetch_all(&mut *conn).await.unwrap();

        return Ok(result);
    }

    /// ## User::retrieve
    ///
    /// Retrieves a user with specified name from the database
    ///
    /// Errors:
    /// + when permission with specified name do not exist
    ///
    pub async fn retrieve(
        conn: &mut PgConnection,
        login: &String,
    ) -> Result<Self, UserRetrieveError> {
        let sql = "SELECT * FROM users WHERE login = $1;";
        let result = query_as(&sql).bind(&login).fetch_one(&mut *conn).await;

        match result {
            Ok(result) => return Ok(result),
            Err(_) => return Err(UserRetrieveError::NotFound),
        };
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
        conn: &mut PgConnection,
        login: &String,
        password: &String,
        details: &Value,
    ) -> Result<(), UserInsertError> {
        let password_hash = match hash_password(password.to_string()) {
            Ok(hash) => hash,
            Err(err) => {
                log_database_interaction(
                    "Inserting a user into database.",
                    json!({ "login": login, "details": details }),
                    DatabaseOperationLogStatus::Err("Password cannot be hashed."),
                );

                return Err(UserInsertError::CannotHash(err.to_string()));
            }
        };

        return Self::insert_unhashed(conn, login, &password_hash, details).await;
    }

    pub async fn insert_unhashed(
        conn: &mut PgConnection,
        login: &String,
        password: &String,
        details: &Value,
    ) -> Result<(), UserInsertError> {
        let sql = "
      INSERT INTO
        users (login, password_hash, details)
      VALUES
        ($1, $2, $3)
      ;
    ";

        let result = query(sql)
            .bind(&login)
            .bind(&password)
            .bind(&details)
            .execute(&mut *conn)
            .await;

        match result {
            Ok(_) => (),
            Err(_) => {
                log_database_interaction(
                    "Inserting a user into database.",
                    json!({ "login": login, "details": details }),
                    DatabaseOperationLogStatus::Err("User with this login already exist."),
                );

                return Err(UserInsertError::NameError);
            }
        };

        log_database_interaction::<String>(
            "Inserting a user into database.",
            json!({ "login": login, "details": details }),
            DatabaseOperationLogStatus::Ok,
        );

        return Ok(());
    }

    /// ## User::delete
    ///
    /// Deletes a user and all of it's related data from the database
    ///
    pub async fn delete(conn: &mut PgConnection, login: String) -> Result<(), UserDeleteError> {
        let sql = "DELETE FROM users_groups WHERE user_login = $1";
        let _ = query(sql).bind(&login).execute(&mut *conn).await;

        let sql = "DELETE FROM login_sessions WHERE user_login = $1";
        let _ = query(sql).bind(&login).execute(&mut *conn).await;

        let sql = "DELETE FROM users WHERE login = $1";
        let result = query(sql).bind(&login).execute(&mut *conn).await.unwrap();

        if result.rows_affected() == 0 {
            log_database_interaction(
                "Deleting user from the database.",
                json!({ "login": login }),
                DatabaseOperationLogStatus::Err("User with this login do not exist."),
            );

            return Err(UserDeleteError::NotFound);
        }

        log_database_interaction::<String>(
            "Deleting user from the database.",
            json!({ "login": login }),
            DatabaseOperationLogStatus::Ok,
        );

        return Ok(());
    }

    /// ## User::login
    ///
    /// Logs in the user with specified session_status, returning the token of created login session
    ///
    /// Errors:
    /// + When the user do not exist
    /// + When the credentials are invalid
    ///
    pub async fn login(
        conn: &mut PgConnection,
        login: &String,
        password: &String,
    ) -> Result<String, UserLoginError> {
        let result = Self::verify_password(conn, login, password).await;

        match result {
            Ok(_) => (),
            Err(err) => match err {
                UserVerifyPasswordError::NotFound => {
                    log_database_interaction(
                        "Inserting user login session to the database.",
                        json!({ "login": login }),
                        DatabaseOperationLogStatus::Err("User with this login do not exist."),
                    );

                    return Err(UserLoginError::NotFound);
                }
                UserVerifyPasswordError::Unauthorized => {
                    log_database_interaction(
                        "Inserting user login session to the database.",
                        json!({ "login": login }),
                        DatabaseOperationLogStatus::Err("Wrong password."),
                    );

                    return Err(UserLoginError::InvalidCredentials);
                }
            },
        }

        let result = LoginSession::insert(conn, login.to_string()).await;

        let token = match result {
            Ok(token) => token,
            Err(err) => match err {
                LoginSessionInsertError::CannotHash(e) => {
                    log_database_interaction(
                        "Inserting user login session to the database.",
                        json!({ "login": login }),
                        DatabaseOperationLogStatus::Err("Cannot hash the token."),
                    );

                    return Err(UserLoginError::CannotHash(e));
                }
                LoginSessionInsertError::UserNotFound => {
                    log_database_interaction(
                        "Inserting user login session to the database.",
                        json!({ "login": login }),
                        DatabaseOperationLogStatus::Err("User with this login do not exist."),
                    );

                    return Err(UserLoginError::InvalidCredentials);
                }
            },
        };

        log_database_interaction::<String>(
            "Inserting user login session to the database.",
            json!({ "login": login }),
            DatabaseOperationLogStatus::Ok,
        );

        return Ok(token);
    }

    /// ## User::has_permission
    ///
    /// Check if a user has a specified permission
    ///
    pub async fn has_permission(
        self: &Self,
        conn: &mut PgConnection,
        permission_name: String,
    ) -> bool {
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
        u.login = $1
      AND
        gp.permission_name  = $2
      LIMIT
        1;
    ";
        let q = query(sql).bind(&self.login).bind(&permission_name);
        let num_rows = q.execute(&mut *conn).await.unwrap().rows_affected();

        if num_rows == 0 {
            return false;
        }

        return true;
    }

    /// ## User::grant_group
    ///
    /// Grants user a group with specified name
    ///
    /// Errors:
    /// + When provided user or group do not exist
    ///
    pub async fn grant_group(
        conn: &mut PgConnection,
        login: &String,
        group_name: &String,
    ) -> Result<(), UserGrantError> {
        if let Err(_) = Group::retrieve(conn, group_name).await {
            log_database_interaction(
                "Granting user a group.",
                json!({ "login": login }),
                DatabaseOperationLogStatus::Err("Group with this name do not exist."),
            );

            return Err(UserGrantError::GroupNotFound);
        }

        if let Err(_) = User::retrieve(conn, login).await {
            log_database_interaction(
                "Granting user a group.",
                json!({ "login": login }),
                DatabaseOperationLogStatus::Err("User with this login do not exist."),
            );

            return Err(UserGrantError::NotFound);
        }

        let sql = "INSERT INTO users_groups (user_login, group_name) VALUES ($1, $2);";
        let result = query(sql)
            .bind(login)
            .bind(group_name)
            .execute(&mut *conn)
            .await;

        log_database_interaction::<String>(
            "Inserting user login session to the database.",
            json!({ "login": login }),
            DatabaseOperationLogStatus::Ok,
        );

        return Ok(());
    }

    /// ## User::revoke_group
    ///
    /// Revokes a group from user with specified login
    ///
    /// Errors:
    /// + When provided user or group do not exist
    ///
    pub async fn revoke_group(
        conn: &mut PgConnection,
        login: &String,
        group_name: &String,
    ) -> Result<(), UserRevokeError> {
        if let Err(_) = Group::retrieve(conn, group_name).await {
            log_database_interaction(
                "Revoking group from a user.",
                json!({ "login": login }),
                DatabaseOperationLogStatus::Err("Group with this name do not exist."),
            );

            return Err(UserRevokeError::GroupNotFound);
        }

        if let Err(_) = User::retrieve(conn, login).await {
            log_database_interaction(
                "Revoking group from a user.",
                json!({ "login": login }),
                DatabaseOperationLogStatus::Err("User with this login do not exist."),
            );

            return Err(UserRevokeError::NotFound);
        }

        let sql = "DELETE FROM users_groups WHERE user_login = $1 AND group_name = $2;";
        let result = query(sql)
            .bind(login)
            .bind(group_name)
            .execute(&mut *conn)
            .await
            .unwrap();

        if result.rows_affected() == 0 {
            log_database_interaction(
                "Revoking group from a user.",
                json!({ "login": login }),
                DatabaseOperationLogStatus::Err("Group was never granted."),
            );

            return Err(UserRevokeError::NotGranted);
        }

        log_database_interaction::<String>(
            "Revoking group from a user.",
            json!({ "login": login }),
            DatabaseOperationLogStatus::Ok,
        );

        return Ok(());
    }

    /// ## User::verify_password
    ///
    /// Retrieves a user and checks a password against it's hash
    ///
    /// Errors:
    /// + when user do not exist
    /// + when the password is invalid
    ///
    pub async fn verify_password(
        db_conn: &mut PgConnection,
        login: &String,
        password: &String,
    ) -> Result<(), UserVerifyPasswordError> {
        let user = match Self::retrieve(db_conn, &login).await {
            Ok(user) => user,
            Err(_) => return Err(UserVerifyPasswordError::NotFound),
        };

        let password_hash =
            &PasswordHash::parse(user.password_hash.as_str(), password_hash::Encoding::B64)
                .unwrap();

        return match Argon2::default().verify_password(password.as_bytes(), password_hash) {
            Ok(_) => Ok(()),
            Err(_) => return Err(UserVerifyPasswordError::Unauthorized),
        };
    }
}

pub fn hash_password(password: String) -> Result<String, String> {
    let pwd = password.as_bytes();
    let salt = SaltString::generate(&mut OsRng);

    let password_hash = match Argon2::default().hash_password(pwd, &salt) {
        Ok(hash) => hash,
        Err(err) => return Err(err.to_string()),
    }
    .to_string();

    return Ok(password_hash);
}
