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
