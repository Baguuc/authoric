use actix_web::{
    delete,
    Responder,
    http::StatusCode, 
    web::{
        Query,
        Data,
        Path
    }
};
use serde::Deserialize;
use serde_json::json;
use crate::{
    config::CauthConfig,
    models::{
        user::User,
        login_session::LoginSession
    },
    web::ServerResponse
};

#[derive(Deserialize)]
struct QueryData {
    session_token: String
}

type PathData = (String, String);

#[delete("/users/{name}/{permission_name}")]
pub async fn controller(
    data: Data<CauthConfig>,
    query: Query<QueryData>,
    path: Path<PathData>
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
