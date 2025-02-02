use std::{
    time::{
        self,
        UNIX_EPOCH
    },
    error::Error
};
use serde_json::Value;
use sqlx::{
    prelude::FromRow,
    query,
    query_as,
    PgConnection
};
use crypto::{
    digest::Digest,
    sha3::Sha3
};
use crate::{
    util::string::json_value_to_pretty_string,
    models::{
        user::{
            User,
            UserVerifyPasswordError
        },
        login_session::{
            LoginSession,
        },
        event::EventCredentials
    }
};

#[derive(FromRow)]
pub struct UserLoginEvent {
    id: i32,
    key: String,
    user_login: String
}

pub enum UserLoginEventRetrieveError {
    /// Returned when the event with specified id cannot be found
    NotFound
}

pub enum UserLoginEventCommitError {
    /// Returned when the key is invalid
    Unauthorized,
    /// Returned when the event with specified id cannot be found
    NotFound,
}

pub enum UserLoginEventInsertError {
    /// Returned when user with login specified in the event's data is not found
    UserNotFound,
    /// Returned when the provided password is invalid
    Unauthorized
}

pub enum UserLoginEventCancelError {
    /// Returned when the key is invalid
    Unauthorized,
    /// Returned when the event with specified id cannot be found
    NotFound
}

impl UserLoginEvent {
    /// ## UserLoginEvent::retrieve
    ///
    /// Retrieves UserLogin event with specifed id
    ///
    /// Errors:
    /// + When the event is not found
    ///
    pub async fn retrieve(
        db_conn: &mut PgConnection,
        id: &i32
    ) -> Result<UserLoginEvent, UserLoginEventRetrieveError> {
        let sql = "
        SELECT
            *
        FROM
            user_login_events
        WHERE
            id = $1;
        ";
        let result = query_as(sql)
            .bind(&id)
            .fetch_one(db_conn)
            .await;

        return match result {
            Ok(event) => Ok(event),
            Err(_) => return Err(UserLoginEventRetrieveError::NotFound),
        };
    }
    
    /// UserLoginEvent::insert
    ///
    /// Inserts a new UserLogin event into database, returning it's key and id
    ///
    /// Errors:
    /// + when the user does not exist
    /// + when the password is invalid
    ///
    pub async fn insert(
        db_conn: &mut PgConnection,
        user_login: &String,
        password: &String
    ) -> Result<EventCredentials, UserLoginEventInsertError> {
        let result = User::verify_password(
            db_conn,
            user_login,
            password
        )
        .await;

        let user = match result {
            Ok(user) => user,
            Err(err) => match err {
                UserVerifyPasswordError::NotFound => return Err(UserLoginEventInsertError::UserNotFound),
                UserVerifyPasswordError::Unauthorized => return Err(UserLoginEventInsertError::Unauthorized),
            }
        };

        let time_since_epoch = time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let key_raw = format!("{}{}", user_login, time_since_epoch);
        
        let mut hasher = Sha3::keccak256();
        hasher.input_str(key_raw.as_str());
        let key = hasher.result_str();
        
        let sql = "
            INSERT INTO 
                user_login_events (key, user_login)
            VALUES
                ($1, $2)
            RETURNING id, key;
        ";

        let result = query_as(sql)
            .bind(&key)
            .bind(&user_login)
            .fetch_one(db_conn)
            .await
            .unwrap();

        return Ok(result);
    }

    /// UserLoginEvent::commit
    ///
    /// Commits the changes in single UserLogin event to the database, returning token
    /// of the created session
    ///
    /// Errors:
    /// + when the event is not found
    /// + when the key is invalid
    /// + when the user with specified login is not found 
    ///
    pub async fn commit(
        db_conn: &mut PgConnection,
        id: &i32,
        key: &String
    ) -> Result<String, UserLoginEventCommitError>
    {
        let retrieved = Self::retrieve(
            db_conn,
            id
        )
        .await;

        let event = match retrieved {
            Ok(event) => event,
            Err(_) => return Err(UserLoginEventCommitError::NotFound)
        };

        if *key != event.key {
            return Err(UserLoginEventCommitError::Unauthorized);
        }

        let token = LoginSession::insert(
            db_conn,
            event.user_login
        ).await
        .unwrap();
        
        let _ = Self::cancel(
            db_conn,
            id,
            key
        )
        .await;

        return Ok(token);
    }

    /// UserLoginEvent::cancel
    ///
    /// Deletes UserLogin event with specified id from the database
    /// 
    /// Errors:
    /// + When the event is not found
    /// + When the key is invalid
    ///
    pub async fn cancel(
        db_conn: &mut PgConnection,
        id: &i32,
        key: &String
    ) ->Result<(), UserLoginEventCancelError>
    {
        let retrieved = Self::retrieve(
            db_conn,
            id
        )
        .await;

        let event = match retrieved {
            Ok(event) => event,
            Err(_) => return Err(UserLoginEventCancelError::NotFound)
        };

        if *key != event.key {
            return Err(UserLoginEventCancelError::Unauthorized);
        }

        let sql = "
        DELETE FROM
            user_login_events
        WHERE
            id = $1;
        ";

        let result = query(sql)
            .bind(&id)
            .execute(db_conn)
            .await
            .unwrap();

        if result.rows_affected() == 0 {
            return Err(UserLoginEventCancelError::NotFound);
        } else {
            return Ok(());
        }
    }
}
