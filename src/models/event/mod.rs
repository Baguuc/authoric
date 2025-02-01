pub mod user_register;
pub mod user_login;
pub mod user_delete;

pub use crate::models::event::{
    user_register::UserRegisterEvent,
    user_login::UserLoginEvent,
    user_delete::UserDeleteEvent
};
use sqlx::prelude::FromRow;
use serde::Serialize;

#[derive(FromRow, Serialize)]
pub struct EventCredentials {
    id: i32,
    key: String
}
