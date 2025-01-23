use std::{
  fmt::Debug,
  time::{
    self,
    UNIX_EPOCH
  }
};
use crypto::{
  digest::Digest,
  sha3::Sha3
};
use serde::{
  Deserialize,
  Serialize
};
use sqlx::{
  query,
  query_as,
  FromRow,PgConnection
};
use crate::{
  models::user::{
    User,
    UserRetrieveError
  },
  util::string::json_value_to_pretty_string
};

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
  pub id: i32,
  pub user_login: String,
  pub status: String,
  pub token: String
}

#[derive(Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct LoginSession {
  pub id: i32,
  pub user_login: String,
  pub status: LoginSessionStatus,
  pub token: String
}

impl ToString for LoginSession {
  fn to_string(&self) -> String {
    let raw = LoginSessionRaw {
      id: self.id.clone(),
      user_login: self.user_login.clone(),
      status: self.status.to_string(),
      token: self.token.clone()
    };
    let formatted = json_value_to_pretty_string(&serde_json::to_value(raw).unwrap());

    return formatted;
  } 
}

#[derive(Debug)]
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
  UserNotFound,
  /// Returned when the token hash cannot be created
  CannotHash(String)
}

impl ToString for LoginSessionInsertError {
  fn to_string(&self) -> String {
    return match self {
      Self::UserNotFound => "Mentioned user not found".to_string(),
      Self::CannotHash(err) => format!("Cannot hash the token. Details;\n{}", err)
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
  /// Selects a user's login session with specified token from the database
  /// 
  pub async fn retrieve(
    conn: &mut PgConnection,
    token: &String
  ) -> Result<Self, LoginSessionRetrieveError> {
    let sql = "
      SELECT 
        *
      FROM
        login_sessions
      WHERE
        token = $1
      AND
        status = 'Commited';
      ;
    ";

    let q = query_as(&sql)
      .bind(&token);

    let raw: LoginSessionRaw = match q.fetch_one(&mut *conn).await {
      Ok(raw) => raw,
      Err(_) => return Err(LoginSessionRetrieveError::NotFound)
    };

    let session = LoginSession {
      id: raw.id,
      user_login: raw.user_login,
      status: LoginSessionStatus::from(raw.status),
      token: raw.token
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
  ) -> Result<String, LoginSessionInsertError> {
    let sql = "
      INSERT INTO
        login_sessions (user_login, status, token)
      VALUES
        ($1, $2, $3)
      RETURNING token;
    ";
    
    let time_since_epoch = match time::SystemTime::now().duration_since(UNIX_EPOCH) {
      Ok(time) => time,
      Err(err) => return Err(LoginSessionInsertError::CannotHash(err.to_string()))
    }
    .as_secs();

    let to_hash = format!("{}{}", &user_login, time_since_epoch);

    let mut hasher = Sha3::keccak256();
    hasher.input_str(to_hash.as_str());
    let token = hasher.result_str();

    let result = query_as(sql)
      .bind(&user_login)
      .bind(status.to_string())
      .bind(&token)
      .fetch_one(&mut *conn)
      .await;

    let row: (String,) = match result {
      Ok(row) => row,
      Err(_) => return Err(LoginSessionInsertError::UserNotFound)
    };
    let token = row.0;

    return Ok(token);
  }


  /// ## LoginSession::delete
  /// 
  /// Deletes a user's login session from the database (e.g. logs out the user)
  /// 
  pub async fn delete(
    conn: &mut PgConnection,
    session_id: i32
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

  /// ## LoginSession::delete_by_token
  /// 
  /// Deletes a user login session from the database
  /// 
  /// Errors:
  /// + When the session is not found
  /// 
  pub async fn delete_by_token(
    conn: &mut PgConnection,
    token: &String
  ) -> Result<(), LoginSessionDeleteError> {
    let sql = "DELETE FROM login_sessions WHERE token = $1;";
    let result = query(sql)
      .bind(&token)
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
    session_id: &i32,
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


  /// ## LoginSession::get_user
  ///
  /// Retrieve a user associated with provided session token
  ///
  /// Errors:
  /// + When a session with specified token do not exist
  /// + When session with provided token is not commited
  ///
  pub async fn get_user(
    conn: &mut PgConnection,
    token: &String
  ) -> Result<User, UserRetrieveError> {
    let sql = "
    SELECT
      u.login,
      u.password_hash,
      u.details
    FROM
      users u
    INNER JOIN
      login_sessions ls
    ON
      u.login = ls.user_login
    WHERE
      ls.token = $1
      AND
      ls.status = 'Commited';
    ";
    let result = query_as(&sql)
      .bind(&token)
      .fetch_one(&mut *conn)
      .await;

    let user = match result {
      Ok(result) => result,
      Err(_) => return Err(UserRetrieveError::NotFound)
    };

    return Ok(user);
  }

  /// ## LoginSession::has_permission
  ///
  /// Check if the user associated with provided token 
  /// has provided permission
  pub async fn has_permission(
    conn: &mut PgConnection,
    token: &String,
    permission_name: &str
  ) -> bool {
    let user = match Self::get_user(
      conn,
      &token
    ).await {
      Ok(user) => user,
      Err(_) => return false
    };

    return user
      .has_permission(
        conn,
        permission_name.to_string()
      )
      .await;
  }
}
