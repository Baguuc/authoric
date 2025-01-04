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
    "permissions:get"
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
    "permissions:post"
  )
  .await;

  if !permitted {
    return ServerResponse::new(
      StatusCode::UNAUTHORIZED,
      None
    );
  }

  if let Err(_) = insert_permission(
    &mut db_conn,
    &json.name,
    &json.description,
    auto_commit
  ).await {
    return ServerResponse::new(
      StatusCode::BAD_REQUEST,
      None
    );
  }

  return ServerResponse::new(
    StatusCode::OK,
    None
  );
}


async fn insert_permission(
  conn: &mut PgConnection, 
  name: &String, 
  description: &String,
  auto_commit: bool
) -> Result<(), PermissionInsertError> {
  if auto_commit {
    Permission::insert(
      conn,
      name,
      description
    )
    .await?;
  } else {
    Permission::event().insert(
      conn,
      name,
      description
    )
    .await;
  }

  return Ok(());
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
    "permissions:delete"
  )
  .await;

  if !permitted {
    return ServerResponse::new(
      StatusCode::UNAUTHORIZED,
      None
    );
  }

  if let Err(_) = del_permission(
    &mut db_conn,
    &name,
    auto_commit
  ).await {
    return ServerResponse::new(
      StatusCode::BAD_REQUEST,
      None
    );
  }

  return ServerResponse::new(
    StatusCode::OK,
    None
  );
}


async fn del_permission(
  conn: &mut PgConnection, 
  name: &String,
  auto_commit: bool
) -> Result<(), PermissionDeleteError> {
  if auto_commit {
    Permission::delete(
      conn,
      name
    )
    .await?;
  } else {
    Permission::event().delete(
      conn,
      name
    )
    .await;
  }

  return Ok(());
}
