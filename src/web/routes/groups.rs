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
    group::{
      Group,
      GroupInsertError,
      GroupDeleteError
    },
    LoginSession,
    Order
  },
  config::CauthConfig,
  web::ServerResponse
};

#[derive(Deserialize)]
struct GetGroupsQueryData {
    session_token: String,
    order_in: Option<Order>,
    page: Option<usize>
}

#[get("/groups")]
pub async fn get_groups(
  query: Query<GetGroupsQueryData>,
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
    "cauth:groups:get"
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
  
  let result = Group::list(
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
struct PostGroupQueryData {
  session_token: String,
  auto_commit: Option<bool>
}

#[derive(Deserialize)]
struct PostGroupJsonData {
  name: String,
  description: String,
  permissions: Vec<String>
}

#[post("/groups")]
pub async fn post_group(
  query: Query<PostGroupQueryData>,
  json: Json<PostGroupJsonData>,
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
    "cauth:groups:post"
  )
  .await;

  if !permitted {
    return ServerResponse::new(
      StatusCode::UNAUTHORIZED,
      None
    );
  }

  if let Err(_) = insert_group(
    &mut db_conn,
    &json.name,
    &json.description,
    &json.permissions,
    auto_commit,
    &query.session_token
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


async fn insert_group(
  conn: &mut PgConnection, 
  name: &String, 
  description: &String,
  permissions: &Vec<String>,
  auto_commit: bool,
  creator_token: &String
) -> Result<(), GroupInsertError> {
  if auto_commit {
    Group::insert(
      conn,
      name,
      description,
      permissions
    )
    .await?;
  } else {
    Group::event().insert(
      conn,
      name,
      description,
      permissions,
      creator_token
    )
    .await;
  }

  return Ok(());
}

#[derive(Deserialize)]
struct DeleteGroupQueryData {
  session_token: String,
  auto_commit: Option<bool>
}

#[delete("/groups/{name}")]
pub async fn delete_group(
  query: Query<DeleteGroupQueryData>,
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
    "cauth:groups:delete"
  )
  .await;

  if !permitted {
    return ServerResponse::new(
      StatusCode::UNAUTHORIZED,
      None
    );
  }

  if let Err(_) = del_group(
    &mut db_conn,
    &name,
    auto_commit,
    &query.session_token
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

async fn del_group(
  conn: &mut PgConnection, 
  name: &String,
  auto_commit: bool,
  creator_token: &String
) -> Result<(), GroupDeleteError> {
  if auto_commit {
    Group::delete(
      conn,
      name
    )
    .await?;
  } else {
    Group::event().delete(
      conn,
      name,
      creator_token
    )
    .await;
  }

  return Ok(());
}
