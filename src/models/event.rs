use std::error::Error;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{prelude::FromRow, query, query_as, PgConnection};

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
  PermissionCreate,
  PermissionDelete,
  GroupCreate,
  GroupDelete,
  UserRegister,
  UserLogin,
  UserDelete
}

impl From<String> for EventType {
  fn from(value: String) -> Self {
    return match value.as_str() {
      "PermissionCreate" => Self::PermissionCreate,
      "PermissionDelete" => Self::PermissionDelete,
      "GroupCreate" => Self::GroupCreate,
      "GroupDelete" => Self::GroupDelete,
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
      Self::PermissionCreate => "PermissionCreate",
      Self::PermissionDelete => "PermissionDelete",
      Self::GroupCreate => "GroupCreate",
      Self::GroupDelete => "GroupDelete",
      Self::UserRegister => "UserRegister",
      Self::UserLogin => "UserLogin",
      Self::UserDelete => "UserDelete"
    }
    .to_string();
  }
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

pub type EventDeleteError = ();

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

  /// ## Event::select
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
  pub async fn insert(conn: &mut PgConnection, _type: EventType, data: Value, creator_token: &String) -> Result<i32, EventInsertError> {
    let sql = "INSERT INTO events (_type, data) VALUES ($1, $2) RETURNING id;";
    let result = query_as(sql)
      .bind(_type.to_string())
      .bind(data)
      .fetch_one(&mut *conn)
      .await;
    let returned_row: (i32,) = result.unwrap();
    let event_id = returned_row.0;

    Self::grant_event_permission(
      conn,
      event_id,
      creator_token
    )
    .await;
    
    return Ok(event_id);
  }

  /// ## Event::delete
  /// 
  /// Cancels the event, deleting it from the database <br>
  /// 
  pub async fn delete(self, conn: &mut PgConnection) -> Result<(), EventDeleteError> {
    let sql = "DELETE FROM events WHERE id = $1";
    let _ = query(sql)
      .bind(&self.id)
      .execute(&mut *conn)
      .await;

    return Ok(());
  }

  /// ## Event::commit
  /// 
  /// Commits a event, posting the changes it should make to the database <br>
  /// 
  pub async fn commit(self: Self, conn: &mut PgConnection) -> Result<(), Box<dyn Error>> {
    let result = match &self._type {
      EventType::PermissionCreate => &self
        .clone()
        .handle_create_permission_event(conn).await,
      EventType::PermissionDelete => &self
        .clone()
        .handle_delete_permission_event(conn).await,
      EventType::GroupCreate => &self
        .clone()
        .handle_create_group_event(conn).await,
      EventType::GroupDelete => &self
        .clone()
        .handle_delete_group_event(conn).await,
      EventType::UserRegister => &self
        .clone()
        .handle_register_user_event(conn).await,
      EventType::UserLogin => &self
        .clone()
        .handle_login_user_event(conn).await,
      EventType::UserDelete => &self
        .clone()
        .handle_delete_user_event(conn).await
    };

    match result {
      Ok(_) => (),
      Err(err) => return Err(err.to_string().into())
    };
    let _ = &self.delete(conn);

    return Ok(());
  }

  /// ## grant_event_permission
  /// 
  /// Grant the created event permission to creator and root
  /// 
  /// Panics:
  /// + this function assumes that the owner_token is validated and user with provided token exists
  ///
  pub async fn grant_event_permission(conn: &mut PgConnection, event_id: i32, owner_token: &String) {
    let owner = LoginSession::retrieve(
      conn, 
      owner_token
    )
    .await
    .unwrap();

    let new_permission_name = format!("cauth:events:use:{}", event_id);
    
    let _ = Permission::insert(
      conn,
      &new_permission_name,
      &format!("Allow the user to interact with event {}", event_id)
    );
    let new_group_name = format!("{}:events", owner.user_login);

    let result = Group::insert(
      conn,
      &new_group_name,
      &format!("Collection of permissions for {} user.", owner.user_login), 
      &vec![new_permission_name.clone()]
    ).await;

    match result {
      Ok(_) => (),
      Err(_) => {
        let _ = Group::grant_permission(
          conn,
          &new_group_name,
          &new_permission_name
        );
      }
    };

    let _ = Group::grant_permission(
      conn, 
      &"root".to_string(), 
      &new_permission_name
    );
  }


  async fn handle_create_permission_event(self, conn: &mut PgConnection) -> Result<(), Box<dyn Error>> {
    let permission = serde_json::from_value::<Permission>(self.data).unwrap();
    
    match Permission::insert(conn, &permission.name, &permission.description).await {
      Ok(_) => (),
      Err(err) => return Err(err.to_string().into())
    };

    return Ok(());
  }

  async fn handle_delete_permission_event(self, conn: &mut PgConnection) -> Result<(), Box<dyn Error>> {
    let permission_name = serde_json::from_value::<String>(self.data).unwrap();
    
    match Permission::delete(conn, &permission_name).await {
      Ok(_) => (),
      Err(err) => return Err(err.to_string().into())
    };

    return Ok(());
  }
  
  async fn handle_create_group_event(self, conn: &mut PgConnection) -> Result<(), Box<dyn Error>> {
    let group = serde_json::from_value::<Group>(self.data).unwrap();
    
    match Group::insert(conn, &group.name, &group.description, &group.permissions).await {
      Ok(_) => (),
      Err(err) => return Err(err.to_string().into()) 
    };

    return Ok(());
  }

  async fn handle_delete_group_event(self, conn: &mut PgConnection) -> Result<(), Box<dyn Error>> {
    let group_name = serde_json::from_value::<String>(self.data).unwrap();
    
    match Group::delete(conn, &group_name).await {
      Ok(_) => (),
      Err(err) => return Err(err.to_string().into()) 
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

