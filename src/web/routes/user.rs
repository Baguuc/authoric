use actix_web::{delete, http::StatusCode, post, web::{Data, Json, Query}, Responder};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::{config::CauthConfig, models::{LoginSession, User}, web::ServerResponse};

#[derive(Deserialize)]
pub struct PostGroupJsonData {
    login: String,
    password: String,
    details: Option<Value>
}

#[post("/users")]
pub async fn post_users(
    _json: Json<PostGroupJsonData>,
    data: Data<CauthConfig>
) -> impl Responder {
  // these will never error
  let mut db_conn = data.db_conn
    .acquire()
    .await
    .unwrap();

  let details = _json.details
    .clone()
    .unwrap_or(json!({}));

  let result = User::insert(
    &mut db_conn, 
    &_json.login, 
    &_json.password, 
    &details
  )
  .await;

  match result {
    Ok(_) => (),
    Err(_) => return ServerResponse::new(
      StatusCode::BAD_REQUEST,
      None
    )
  };

  return ServerResponse::new(
    StatusCode::OK,
    None
  );
}
