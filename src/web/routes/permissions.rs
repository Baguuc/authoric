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
struct GetPermissionsQueryData {
    session_token: String,
    order_in: Option<Order>,
    page: Option<usize>
}

#[get("/permissions")]
pub async fn get_permissions(
  query: Query<GetPermissionsQueryData>,
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
    "cauth:permissions:get"
  )
  .await;

  if !permitted {
    return ServerResponse::new(
      StatusCode::UNAUTHORIZED,
      None
    );
  }

  let mut db_conn = data.db_conn
    .acquire()
    .await
    .unwrap();
  
  let result = Permission::list(
    &mut db_conn,
    query.order_in,
    Some(query.page.unwrap_or(0) * 10),
    Some(10)
  )
  .await
  .unwrap();

  return ServerResponse::new(
    StatusCode::OK,
    Some(json!(result))
  );
}

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

  return insert_permission(
    &mut db_conn,
    &json.name,
    &json.description,
    auto_commit,
    &query.session_token
  ).await;
}


async fn insert_permission(
  conn: &mut PgConnection, 
  name: &String, 
  description: &String,
  auto_commit: bool,
  creator_token: &String
) -> ServerResponse {
  if auto_commit {
    let result = Permission::insert(
      conn,
      name,
      description
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
      conn,
      name,
      description,
      &creator_token
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

#[delete("/permissions")]
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

  return del_permission(
    &mut db_conn,
    &name,
    auto_commit,
    &query.session_token
  ).await
}


async fn del_permission(
  conn: &mut PgConnection, 
  name: &String,
  auto_commit: bool,
  creator_token: &String
) -> ServerResponse {
   if auto_commit {
    let result = Permission::delete(
      conn,
      name
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
      conn,
      name,
      &creator_token
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
