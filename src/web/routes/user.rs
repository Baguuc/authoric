use actix_web::{delete, get, http::StatusCode, post, web::{Data, Json, Path, Query}, Responder};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::{config::CauthConfig, models::{login_session::LoginSessionStatus, LoginSession, User}, web::ServerResponse};

#[delete("/users/{name}/{permission_name}")]
pub async fn revoke_group(
    data: Data<CauthConfig>,
    path: Path<(String, String)>
) ->impl Responder {
  // these will never error
  let mut db_conn = data.db_conn
    .begin()
    .await
    .unwrap();

  let permitted = LoginSession::has_permission(
    &mut db_conn,
    &query.session_token,
    "cauth:users:update"
  )
  .await;

  if !permitted {
    return ServerResponse::new(
      StatusCode::UNAUTHORIZED,
      None
    );
  }

  let (user_login, group_name) = path.into_inner();

  let result = User::revoke_group(
    &mut db_conn,
    &user_login,
    &group_name
  )
  .await;

  match result {
      Ok(_) => (),
      Err(_) => {
          return ServerResponse::new(
            StatusCode::BAD_REQUEST,
            None
          )
      }
  };

  match db_conn.commit().await {
    Ok(_) => (),
    Err(err) => {
      eprintln!("Error committing changes: {}", err);
    }
  };

  return ServerResponse::new(
    StatusCode::OK,
    None
  );
}
