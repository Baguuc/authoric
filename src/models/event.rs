use std::{
    time::{
        self,
        UNIX_EPOCH
    },
    error::Error
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{prelude::FromRow, query, query_as, PgConnection};
use crypto::{
  digest::Digest,
  sha3::Sha3
};
use crate::util::string::json_value_to_pretty_string;
use super::{group::Group, login_session::{LoginSession, LoginSessionStatus}, permission::Permission, user::User, Order};

#[derive(FromRow, Serialize, Deserialize)]
pub struct EventRaw {
  id: i32,
  _type: String,
  data: Value
}

#[derive(Clone, PartialEq, Eq)]
pub enum EventType {
  UserRegister,
  UserLogin,
  UserDelete
}

impl From<String> for EventType {
  fn from(value: String) -> Self {
    return match value.as_str() {
      "UserRegister" => Self::UserRegister,
      "UserLogin" => Self::UserLogin,
      "UserDelete" => Self::UserDelete,
      _ => todo!()
    };
  }
}

impl ToString for EventType {
  fn to_string(self: &Self) -> String {
    return match self {
      Self::UserRegister => "UserRegister",
      Self::UserLogin => "UserLogin",
      Self::UserDelete => "UserDelete"
    }
    .to_string();
  }
}

#[derive(Serialize)]
pub struct EventCredentials {
    id: i32,
    key: String
}

#[derive(Clone, PartialEq, Eq)]
pub struct Event {
  id: i32,
  _type: EventType,
  data: Value
}

impl ToString for Event {
  fn to_string(&self) -> String {
    let raw = EventRaw {
      id: self.id,
      _type: self._type.to_string(),
      data: self.data.clone()
    };
    let formatted = json_value_to_pretty_string(&serde_json::to_value(&raw).unwrap());

    return formatted;
  }
}

pub type EventListError = ();

#[derive(Debug)]
pub enum EventRetrieveError {
  /// Returned when a event with specified id is not found
  NotFound
}

impl ToString for EventRetrieveError {
  fn to_string(&self) -> String {
    return match self {
      Self::NotFound => "This event cannot be found".to_string()
    }
  }
}

pub type EventInsertError = ();

pub enum EventCommitError {
    /// Returned when the event is not found
    NotFound,
    /// Returned when the key - id pair is wrong
    Unauthorized,
    /// Returned when the data inside the event is wrong
    CannotCommit
}

pub enum EventDeleteError {
    /// Returned when the key - id pair is invalid
    /// or the event do not exist
    CannotInteract
}

impl Event {
  /// ## Event::list
  /// 
  /// Lists number of events in specified order with specified offset from the database
  /// 
  pub async fn list(
    conn: &mut PgConnection,
    order: Option<Order>,
    offset: Option<usize>,
    limit: Option<usize>
  ) -> Result<Vec<Self>, EventListError> {
    let order = order.unwrap_or(Order::Ascending);
    let offset = offset.unwrap_or(0);
    let limit = limit.unwrap_or(10);
    
    let sql = format!(
      "SELECT * FROM events ORDER BY id {} OFFSET {} ROWS LIMIT {};",
      order.to_string(),
      offset,
      limit
    );
    let result: Vec<EventRaw> = query_as(&sql)
      .fetch_all(&mut *conn)
      .await
      .unwrap();

    let events = result
      .iter()
      .map(|row| {
        return Event {
          id: row.id,
          _type: EventType::from(row._type.clone()),
          data: row.data.clone()
        };
      })
      .collect::<Vec<Event>>();

    return Ok(events);
  }

  /// ## Event::retrieve
  /// 
  /// Retrieves a event with specified id from the database
  /// 
  /// Errors:
  /// + When a event with specified id do not exist
  /// 
  pub async fn retrieve(conn: &mut PgConnection, with_id: i32) -> Result<Self, EventRetrieveError> {
    let sql = "SELECT * FROM events WHERE id = $1;";
    let result = query_as(sql)
      .bind(&with_id)
      .fetch_one(&mut *conn)
      .await;

    let event_raw: EventRaw = match result {
      Ok(event) => event,
      Err(_) => return Err(EventRetrieveError::NotFound)
    };

    let event = Event {
      id: event_raw.id,
      _type: EventType::from(event_raw._type),
      data: event_raw.data
    };

    return Ok(event);
  }

  /// ## Event::insert
  /// 
  /// Inserts a event with provided data into the database and uses grant_event_permission function on it <br>
  /// 
  pub async fn insert(conn: &mut PgConnection, _type: EventType, data: Value) -> Result<EventCredentials, EventInsertError> {
    let _type = _type.to_string();
    let time_since_epoch = time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let key_raw = format!("{}{}", _type, time_since_epoch);
    
    let mut hasher = Sha3::keccak256();
    hasher.input_str(key_raw.as_str());
    let key = hasher.result_str();

    let sql = "INSERT INTO events (_type, data, key) VALUES ($1, $2) RETURNING id;";
    let result = query_as(sql)
      .bind(&_type.to_string())
      .bind(&data)
      .bind(&key)
      .fetch_one(&mut *conn)
      .await;
    let returned_row: (i32,) = result.unwrap();
    let event_id = returned_row.0;
    
    return Ok(EventCredentials {
        id: event_id,
        key
    });
  }

  /// ## Event::delete
  /// 
  /// Cancels the event, deleting it from the database <br>
  /// 
  pub async fn delete(conn: &mut PgConnection, id: i32, key: &String) -> Result<(), EventDeleteError> {
    let sql = "DELETE FROM events WHERE id = $1 AND key = $2";
    let result = query(sql)
      .bind(&id)
      .bind(&key)
      .execute(&mut *conn)
      .await
      .unwrap();

    if result.rows_affected() == 0 {
        return Err(EventDeleteError::CannotInteract);
    }

    return Ok(());
  }

  /// ## Event::commit
  /// 
  /// Commits a event, posting the changes it should make to the database <br>
  /// 
  pub async fn commit(conn: &mut PgConnection, id: i32, key: &String) -> Result<(), EventCommitError> {
    let event = Self::retrieve(conn, id).await;
    let event = match event {
        Ok(event) => event,
        Err(_) => return Err(EventCommitError::NotFound)
    };

    let result = match &event._type {
      EventType::UserRegister => &event
        .clone()
        .handle_register_user_event(conn).await,
      EventType::UserLogin => &event
        .clone()
        .handle_login_user_event(conn).await,
      EventType::UserDelete => &event
        .clone()
        .handle_delete_user_event(conn).await
    };

    match result {
      Ok(_) => (),
      Err(err) => return Err(EventCommitError::CannotCommit)
    };
    
    match Event::delete(conn, id, key).await {
        Ok(_) => (),
        // at this point if we retrieved it before,
        // we know only Unauthorized can occur
        Err(_) => return Err(EventCommitError::Unauthorized)
    };

    return Ok(());
  }

  async fn handle_register_user_event(self, conn: &mut PgConnection) -> Result<(), Box<dyn Error>> {
    let user = serde_json::from_value::<User>(self.data).unwrap();

    match User::insert(conn, &user.login, &user.password_hash, &user.details).await {
      Ok(()) => (),
      Err(err) => return Err(err.to_string().into())
    };

    return Ok(());
  }

  async fn handle_login_user_event(self, conn: &mut PgConnection) -> Result<(), Box<dyn Error>> {
    let session_id = serde_json::from_value::<i32>(self.data).unwrap();

    let _ = LoginSession::update(
      conn,
      &session_id,
      LoginSessionStatus::Commited
    );

    return Ok(());
  }

  async fn handle_delete_user_event(self, conn: &mut PgConnection) -> Result<(), Box<dyn Error>> {
    let user_login = serde_json::from_value::<String>(self.data).unwrap();

    let _ = User::delete(conn, user_login).await;

    return Ok(());
  }
}

