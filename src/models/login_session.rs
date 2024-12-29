
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as, FromRow, PgConnection};

use crate::util::string::json_value_to_pretty_string;

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

impl ToString for LoginSession {
  fn to_string(&self) -> String {
    let raw = LoginSessionRaw {
      id: self.id.clone(),
      user_login: self.user_login.clone(),
      status: self.status.to_string()
    };
    let formatted = json_value_to_pretty_string(&serde_json::to_value(raw).unwrap());

    return formatted;
  } 
}

pub enum LoginSessionRetrieveError {
  /// Returned when the session is not found
  NotFound
}

impl ToString for LoginSessionRetrieveError {
  fn to_string(&self) -> String {
    return match self {
      Self::NotFound => "Login session not found".to_string()
    }
  }
}

#[derive(Debug)]
pub enum LoginSessionInsertError {
  /// Returned when the user attached to the session does not exist
  UserNotFound
}

impl ToString for LoginSessionInsertError {
  fn to_string(&self) -> String {
    return match self {
      Self::UserNotFound => "Mentioned user not found".to_string()
    }
  }
}

pub enum LoginSessionDeleteError {
  /// Returned when the session wasn't deleted because it never existed
  NotFound
}

impl ToString for LoginSessionDeleteError {
  fn to_string(&self) -> String {
    return match self {
      Self::NotFound => "Login session not found".to_string()
    }
  }
}

type LoginSessionUpdateError = ();

impl LoginSession {
  /// ## LoginSession::retrieve
  /// 
  /// Selects a user's loggin session with specified id from the database
  /// 
  pub async fn retrieve(
    conn: &mut PgConnection,
    id: i64
  ) -> Result<Self, LoginSessionRetrieveError> {
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

    let raw: LoginSessionRaw = match q.fetch_one(&mut *conn).await {
      Ok(raw) => raw,
      Err(_) => return Err(LoginSessionRetrieveError::NotFound)
    };

    let session = LoginSession {
      id: raw.id,
      user_login: raw.user_login,
      status: LoginSessionStatus::from(raw.status)
    };

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
    conn: &mut PgConnection,
    user_login: String,
    status: LoginSessionStatus
  ) -> Result<i64, LoginSessionInsertError> {
    let sql = "
      INSERT INTO
        login_sessions (user_login, status)
      VALUES
        ($1, $2)
      RETURNING id;
      ;
    ";

    let result = query_as(sql)
      .bind(&user_login)
      .bind(status.to_string())
      .fetch_one(&mut *conn)
      .await;

    let row: (i64,) = match result {
      Ok(row) => row,
      Err(_) => return Err(LoginSessionInsertError::UserNotFound)
    };
    let session_id = row.0;

    return Ok(session_id);
  }


  /// ## LoginSession::delete
  /// 
  /// Deletes a user's login session from the database (e.g. logs out the user)
  /// 
  pub async fn delete(
    conn: &mut PgConnection,
    session_id: i64
  ) -> Result<(), LoginSessionDeleteError> {
    let sql = "DELETE FROM login_sessions WHERE id = $1";
    let result = query(sql)
      .bind(&session_id)
      .execute(&mut *conn)
      .await;

    let rows_affected = result
      .unwrap()
      .rows_affected();

    if rows_affected == 0 {
      return Err(LoginSessionDeleteError::NotFound)
    }

    return Ok(());
  }
  
  /// ## LoginSession::update
  /// 
  /// Updates a login session with new status
  /// 
  pub async fn update(
    conn: &mut PgConnection,
    session_id: &i64,
    new_status: LoginSessionStatus
  ) -> Result<(), LoginSessionUpdateError> {
    let sql = "UPDATE login_sessions SET status = $1 WHERE id = $2;";
    let result = query(sql)
      .bind(new_status.to_string())
      .bind(session_id)
      .execute(&mut *conn)
      .await;
    
    return Ok(());
  }
}