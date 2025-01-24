use actix_web::{delete, get, http::StatusCode, post, web::{Data, Json, Path, Query}, Responder};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::{config::CauthConfig, models::{login_session::LoginSessionStatus, LoginSession, User}, web::ServerResponse};

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
    .begin()
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
 
    match db_conn.commit().await {
        Ok(_) => (),
        Err(err) => {
            eprintln!("Error committing changes: {}", err);
        }
    };

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
        
        match db_conn.commit().await {
            Ok(_) => (),
            Err(err) => {
                eprintln!("Error committing changes: {}", err);
            }
        };

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
    .begin()
    .await
    .unwrap();

    let result = LoginSession::delete_by_token(
      &mut db_conn,
      &query.session_token
  )
  .await;

  match db_conn.commit().await {
    Ok(_) => (),
    Err(err) => {
      eprintln!("Error committing changes: {}", err);
    }
  };

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



#[get("/user/permissions/{permission_name}")]
pub async fn get_user_permissions(
    permission_name: Path<String>,
    query: Query<DeleteUserQueryData>,
    data: Data<CauthConfig>
) -> impl Responder {
  // these will never error
  let mut db_conn = data.db_conn
    .acquire()
    .await
    .unwrap();

  let result = LoginSession::has_permission(
    &mut db_conn,
    &query.session_token,
    &permission_name.into_inner()
  )
  .await;

  return ServerResponse::new(
    StatusCode::OK,
    Some(json!({
        "has": result
    }))
  );
}

#[post("/users/{name}/{permission_name}")]
pub async fn grant_group(
    data: Data<CauthConfig>,
    path: Path<(String, String)>
) -> impl Responder {
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

  let result = User::grant_group(
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
