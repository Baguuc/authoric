use actix_web::{
  Responder,
  get,
  web::{
    Query,
    Data
  },
  http::StatusCode
};
use serde::Deserialize;
use serde_json::json;
use crate::{
  models::{
    Permission,
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
async fn get_permissions(
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
