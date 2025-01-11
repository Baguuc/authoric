use actix_web::{delete, http::StatusCode, post, web::{Data, Json, Path, Query}, Responder};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::{config::CauthConfig, models::{login_session::LoginSessionStatus, LoginSession, User}, web::ServerResponse};

#[derive(Deserialize)]
pub struct PostUsersJsonData {
    login: String,
    password: String,
    details: Option<Value>
}

#[post("/users")]
pub async fn post_users(
    _json: Json<PostUsersJsonData>,
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
    Ok(_) => return ServerResponse::new(
      StatusCode::OK,
      None
    ),
    Err(_) => return ServerResponse::new(
      StatusCode::BAD_REQUEST,
      None
    )
  };
}

#[derive(Deserialize)]
pub struct DeleteUsersQueryData {
    session_token: String,
    auto_commit: bool
}

#[delete("/users/{login}")]
pub async fn delete_users(
    query: Query<DeleteUsersQueryData>,
    name: Path<String>,
    data: Data<CauthConfig>
) -> impl Responder {
  // these will never error
  let mut db_conn = data.db_conn
    .acquire()
    .await
    .unwrap();

  let permitted = LoginSession::has_permission(
    &mut db_conn,
    &query.session_token,
    &format!("users:delete:{}", name)
  )
  .await;
  
  if !permitted {
    return ServerResponse::new(
      StatusCode::UNAUTHORIZED,
      None
    );
  }

  if query.auto_commit {
    let result = User::delete(
      &mut db_conn,
      (&name).to_string()
    )
    .await;

    match result {
      Ok(_) => return ServerResponse::new(
        StatusCode::OK,
        None
      ),
      Err(_) => return ServerResponse::new(
        StatusCode::BAD_REQUEST,
        None
      )
    }
  } else {
    let result = User::event().delete(
      &mut db_conn,
      (&name).to_string(),
      &query.session_token
    )
    .await;

    match result {
      Ok(event_id) => return ServerResponse::new(
        StatusCode::OK,
        Some(json!({
          "event_id": event_id
        }))
      ),
      Err(_) => return ServerResponse::new(
        StatusCode::BAD_REQUEST,
        None
      )
    }
  }
}

#[derive(Deserialize)]
struct GetUserQueryData {
  session_token: String
}

#[post("/users")]
pub async fn get_user(
  query: Query<GetUserQueryData>,
  data: Data<CauthConfig>
) -> impl Responder {
  // these will never error
  let mut db_conn = data.db_conn
    .acquire()
    .await
    .unwrap();

  let result = LoginSession::retrieve_user(
    &mut db_conn,
    &query.session_token
  )
  .await;

  match result {
    Ok(user) => return ServerResponse::new(
      StatusCode::OK,
      Some(json!(user))
    ),
    Err(_) => return ServerResponse::new(
      StatusCode::BAD_REQUEST,
      None
    )
  };
}

#[derive(Deserialize)]
pub struct PostUserQueryData {
  auto_commit: Option<bool>
}

#[derive(Deserialize)]
pub struct PostUserJsonData {
  login: String,
  password: String,
}

#[post("/user")]
pub async fn post_user(
    query: Query<PostUserQueryData>,
    json: Json<PostUserJsonData>,
    data: Data<CauthConfig>
) -> impl Responder {
  // these will never error
  let mut db_conn = data.db_conn
    .acquire()
    .await
    .unwrap();

  let auto_commit = query.auto_commit
    .unwrap_or(true);

  if auto_commit {
    let result = User::login(
        &mut db_conn,
        &json.login,
        &json.password,
        LoginSessionStatus::Commited
    )
    .await;

    match result {
        Ok(token) => return ServerResponse::new(
          StatusCode::OK,
          Some(json!({
              "token": token
          }))
        ),
        Err(_) => return ServerResponse::new(
            StatusCode::BAD_REQUEST,
            None
        )
      };
    } else {
        let result = User::event().login(
            &mut db_conn, 
            &json.login,
            &json.password
        )
        .await;

        match result {
          Ok(event_id) => return ServerResponse::new(
            StatusCode::OK,
            Some(json!({
              "event_id": event_id
            }))
          ),
          Err(_) => return ServerResponse::new(
            StatusCode::BAD_REQUEST,
            None
          )
        }
    }
}

#[derive(Deserialize)]
pub struct DeleteUserQueryData {
    session_token: String
}

#[delete("/user")]
pub async fn delete_user(
    query: Query<DeleteUserQueryData>,
    data: Data<CauthConfig>
) -> impl Responder {
  // these will never error
  let mut db_conn = data.db_conn
    .acquire()
    .await
    .unwrap();

    let result = User::delete_by_token(
      &mut db_conn,
      &query.session_token
  )
  .await;

  match result {
    Ok(token) => return ServerResponse::new(
      StatusCode::OK,
      Some(json!({
          "token": token
      }))
    ),
      Err(_) => return ServerResponse::new(
        StatusCode::BAD_REQUEST,
        None
    )
  };
}