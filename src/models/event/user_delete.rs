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
            UserRetrieveError
        },
        event::EventCredentials
    }
};

#[derive(FromRow)]
pub struct UserDeleteEvent {
    id: i32,
    key: String,
    user_login: String
}

pub enum UserDeleteEventRetrieveError {
    /// Returned when the event with specified id cannot be found
    NotFound
}

pub enum UserDeleteEventCommitError {
    /// Returned when the key is invalid
    Unauthorized,
    /// Returned when the event with specified id cannot be found
    NotFound,
}

pub enum UserDeleteEventInsertError {
    /// Returned when user with login specified in the event's data is not found
    UserNotFound,
}

pub enum UserDeleteEventCancelError {
    /// Returned when the key is invalid
    Unauthorized,
    /// Returned when the event with specified id cannot be found
    NotFound
}

impl UserDeleteEvent {
    /// ## UserDeleteEvent::retrieve
    ///
    /// Retrieves UserDelete event with specifed id
    ///
    /// Errors:
    /// + When the event is not found
    ///
    pub async fn retrieve(
        db_conn: &mut PgConnection,
        id: &i32
    ) -> Result<UserDeleteEvent, UserDeleteEventRetrieveError> {
        let sql = "
        SELECT
            *
        FROM
            user_delete_events
        WHERE
            id = $1;
        ";
        let result = query_as(sql)
            .bind(&id)
            .fetch_one(db_conn)
            .await;

        return match result {
            Ok(event) => Ok(event),
            Err(_) => return Err(UserDeleteEventRetrieveError::NotFound),
        };
    }
    
    /// UserDeleteEvent::insert
    ///
    /// Inserts a new UserDelete event into database, returning it's key and id
    ///
    /// Errors:
    /// + when the user does not exist
    /// + when the password is invalid
    ///
    pub async fn insert(
        db_conn: &mut PgConnection,
        user_login: &String
    ) -> Result<EventCredentials, UserDeleteEventInsertError> {
        let result = User::retrieve(
            db_conn,
            user_login
        )
        .await;

        let user = match result {
            Ok(user) => user,
            Err(err) => match err {
                UserRetrieveError::NotFound => return Err(UserDeleteEventInsertError::UserNotFound)
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
                user_delete_events (key, user_login)
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

    /// UserDeleteEvent::commit
    ///
    /// Commits the changes in single UserDelete event to the database
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
    ) -> Result<(), UserDeleteEventCommitError>
    {
        let retrieved = Self::retrieve(
            db_conn,
            id
        )
        .await;

        let event = match retrieved {
            Ok(event) => event,
            Err(_) => return Err(UserDeleteEventCommitError::NotFound)
        };

        if *key != event.key {
            return Err(UserDeleteEventCommitError::Unauthorized);
        }

        let result = User::delete(
            db_conn,
            event.user_login
        ).await;
        
        let _ = Self::cancel(
            db_conn,
            id,
            key
        )
        .await;

        return match result {
            Ok(_) => Ok(()),
            Err(_) => return Err(UserDeleteEventCommitError::NotFound)
        };
    }

    /// UserDeleteEvent::cancel
    ///
    /// Deletes UserDelete event with specified id from the database
    /// 
    /// Errors:
    /// + When the event is not found
    /// + When the key is invalid
    ///
    pub async fn cancel(
        db_conn: &mut PgConnection,
        id: &i32,
        key: &String
    ) ->Result<(), UserDeleteEventCancelError>
    {
        let retrieved = Self::retrieve(
            db_conn,
            id
        )
        .await;

        let event = match retrieved {
            Ok(event) => event,
            Err(_) => return Err(UserDeleteEventCancelError::NotFound)
        };

        if *key != event.key {
            return Err(UserDeleteEventCancelError::Unauthorized);
        }

        let sql = "
        DELETE FROM
            user_delete_events
        WHERE
            id = $1;
        ";

        let result = query(sql)
            .bind(&id)
            .execute(db_conn)
            .await
            .unwrap();

        if result.rows_affected() == 0 {
            return Err(UserDeleteEventCancelError::NotFound);
        } else {
            return Ok(());
        }
    }
}
