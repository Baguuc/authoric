use actix_web::{
  delete, 
  get, 
  http::StatusCode, 
  post, 
  web::{
    Data, Json, Path, Query
  }, Responder
};
use serde::Deserialize;
use serde_json::json;
use sqlx::PgConnection;
use crate::{
  models::{
    permission::{
      Permission,
      PermissionInsertError,
      PermissionDeleteError
    },
    LoginSession,
    Order
  },
  config::CauthConfig,
  web::ServerResponse
};

#[derive(Deserialize)]
struct PostPermissionQueryData {
  session_token: String,
  auto_commit: Option<bool>
}

#[derive(Deserialize)]
struct PostPermissionJsonData {
  name: String,
  description: String
}

#[post("/permissions")]
pub async fn post_permission(
  query: Query<PostPermissionQueryData>,
  json: Json<PostPermissionJsonData>,
  data: Data<CauthConfig>
) -> impl Responder {
  // these will never error
  let mut db_conn = data.db_conn
    .acquire()
    .await
    .unwrap();

  let auto_commit = query
    .auto_commit
    .unwrap_or(true);

  let permitted = LoginSession::has_permission(
    &mut db_conn,
    &query.session_token,
    "cauth:permissions:post"
  )
  .await;

  if !permitted {
    return ServerResponse::new(
      StatusCode::UNAUTHORIZED,
      None
    );
  }
  
  if auto_commit {
    let result = Permission::insert(
      &mut db_conn,
      &json.name,
      &json.description
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
    let result = Permission::event().insert(
      &mut db_conn,
      &json.name,
      &json.description,
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
      Err(_) =>  return ServerResponse::new(
        StatusCode::BAD_REQUEST,
        None
      ),
    }
  }
}

#[derive(Deserialize)]
struct DeletePermissionQueryData {
  session_token: String,
  auto_commit: Option<bool>
}

#[delete("/permissions/{name}")]
pub async fn delete_permission(
  query: Query<DeletePermissionQueryData>,
  data: Data<CauthConfig>,
  name: Path<String>
) -> impl Responder {
  // these will never error
  let mut db_conn = data.db_conn
    .acquire()
    .await
    .unwrap();

  let auto_commit = query
    .auto_commit
    .unwrap_or(true);

  let permitted = LoginSession::has_permission(
    &mut db_conn,
    &query.session_token,
    "cauth:permissions:delete"
  )
  .await;

  if !permitted {
    return ServerResponse::new(
      StatusCode::UNAUTHORIZED,
      None
    );
  }

  if auto_commit {
    let result = Permission::delete(
      &mut db_conn,
      &name.into_inner()
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
    let result = Permission::event().delete(
      &mut db_conn,
      &name.into_inner(),
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
      Err(_) =>  return ServerResponse::new(
        StatusCode::BAD_REQUEST,
        None
      ),
    }
  }
}
