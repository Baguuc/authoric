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
            hash_password
        },
        event::EventCredentials
    }
};

#[derive(FromRow)]
pub struct UserRegisterEvent {
    id: i32,
    key: String,
    user_login: String,
    password_hash: String,
    details: serde_json::Value
}

pub enum UserRegisterEventRetrieveError {
    /// Returned when the event with specified id cannot be found
    NotFound
}

pub enum UserRegisterEventInsertError {
    /// Returned when the mentioned user already exists
    AlreadyExists,
    /// Returned when the password cannot be hashed
    CannotHash(String)
}

pub enum UserRegisterEventCommitError {
    /// Returned when the key is invalid
    Unauthorized,
    /// Returned when the event with specified id cannot be found
    NotFound
}

pub enum UserRegisterEventCancelError {
    /// Returned when the key is invalid
    Unauthorized,
    /// Returned when the event with specified id cannot be found
    NotFound
}

impl UserRegisterEvent {
    /// ## UserRegisterEvent::retrieve
    ///
    /// Retrieves UserRegister event with specifed id
    ///
    /// Errors:
    /// + When the event is not found
    ///
    pub async fn retrieve(
        db_conn: &mut PgConnection,
        id: &i32
    ) -> Result<UserRegisterEvent, UserRegisterEventRetrieveError> {
        let sql = "
        SELECT
            *
        FROM
            user_register_events
        WHERE
            id = $1;
        ";
        let result = query_as(sql)
            .bind(&id)
            .fetch_one(db_conn)
            .await;

        return match result {
            Ok(event) => Ok(event),
            Err(_) => return Err(UserRegisterEventRetrieveError::NotFound),
        };
    }
    
    /// UserRegisterEvent::insert
    ///
    /// Inserts a new UserRegister event into database, returning it's key and id
    ///
    /// Errors:
    /// + when the user with specified login already exists
    pub async fn insert(
        db_conn: &mut PgConnection,
        user_login: &String,
        password: &String,
        details: &serde_json::Value
    ) -> Result<EventCredentials, UserRegisterEventInsertError> {
        let user = User::retrieve(
            db_conn,
            user_login
        )
        .await;

        match user {
            // found
            Ok(_) => return Err(UserRegisterEventInsertError::AlreadyExists),
            // not found
            Err(_) => ()
        };

        let time_since_epoch = time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let key_raw = format!("{}{}", user_login, time_since_epoch);
        
        let mut hasher = Sha3::keccak256();
        hasher.input_str(key_raw.as_str());
        let key = hasher.result_str();

        let password_hash = match hash_password(password.clone()) {
            Ok(hash) => hash,
            Err(err) => return Err(UserRegisterEventInsertError::CannotHash(err))
        };

        let sql = "
            INSERT INTO 
                user_register_events (key, user_login, password_hash, details)
            VALUES
                ($1, $2, $3, $4)
            RETURNING id, key;
        ";

        let result = query_as(sql)
            .bind(&key)
            .bind(&user_login)
            .bind(&password_hash)
            .bind(&details)
            .fetch_one(db_conn)
            .await
            .unwrap();

        return Ok(result);
    }

    /// UserRegisterEvent::commit
    ///
    /// Commits the changes in single UserRegister event to the database
    ///
    /// Errors:
    /// + when the event is not found
    /// + when the key is invalid
    ///
    pub async fn commit(
        db_conn: &mut PgConnection,
        id: &i32,
        key: &String
    ) -> Result<(), UserRegisterEventCommitError>
    {
        let retrieved = Self::retrieve(
            db_conn,
            id
        )
        .await;

        let event = match retrieved {
            Ok(event) => event,
            Err(_) => return Err(UserRegisterEventCommitError::NotFound)
        };

        if *key != event.key {
            return Err(UserRegisterEventCommitError::Unauthorized);
        }

        let token = User::insert_unhashed(
            db_conn,
            &event.user_login,
            &event.password_hash,
            &event.details
        )
        .await
        // we know the user is unique already is it won't panic
        .unwrap();
        
        let _ = Self::cancel(
            db_conn,
            id,
            key
        )
        .await;

        return Ok(token);
    }

    /// UserRegisterEvent::cancel
    ///
    /// Deletes UserRegister event with specified id from the database
    /// 
    /// Errors:
    /// + When the event is not found
    ///
    pub async fn cancel(
        db_conn: &mut PgConnection,
        id: &i32,
        key: &String
    ) ->Result<(), UserRegisterEventCancelError>
    {
        let retrieved = Self::retrieve(
            db_conn,
            id
        )
        .await;

        let event = match retrieved {
            Ok(event) => event,
            Err(_) => return Err(UserRegisterEventCancelError::NotFound)
        };

        if *key != event.key {
            return Err(UserRegisterEventCancelError::Unauthorized);
        }

        let sql = "
        DELETE FROM
            user_register_events
        WHERE
            id = $1;
        ";

        let result = query(sql)
            .bind(&id)
            .execute(db_conn)
            .await
            .unwrap();

        if result.rows_affected() == 0 {
            return Err(UserRegisterEventCancelError::NotFound);
        } else {
            return Ok(());
        }
    }
}
