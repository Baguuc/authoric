pub mod permission;
pub mod group;
pub mod user;
pub mod login_session;
pub mod event;

use serde::Deserialize;
use sqlx::PgConnection;
pub use crate::models::{
    permission::Permission,
    group::Group,
    user::User,
    login_session::LoginSession,
    event::Event
};

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum Order {
  Ascending,
  Descending
}


impl ToString for Order {
  fn to_string(&self) -> String {
    return match self {
      Order::Ascending => "ASC".to_string(),
      Order::Descending => "DESC".to_string(),
    };
  }
}


  /// ## grant_event_permission
  /// 
  /// Grant the created event permission to creator and root
  /// 
  /// Panics:
  /// + this function assumes that the owner_token is validated and user with provided token exists
  ///
pub async fn grant_event_permission(conn: &mut PgConnection, event_id: i64, owner_token: &String) {
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
  let _ = Group::insert(
    conn,
    &format!("{}:events", owner.user_login), 
    &format!("Collection of permissions for {} user.", owner.user_login), 
    &vec![new_permission_name.clone()]
  );
  let _ = Group::grant_permission(
    conn, 
    &"root".to_string(), 
    &new_permission_name
  );
}
