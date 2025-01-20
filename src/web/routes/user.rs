use actix_web::{delete, get, http::StatusCode, post, web::{Data, Json, Path, Query}, Responder};
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
    .begin()
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
    }
  } else { 
    let result = User::event().delete(
      &mut db_conn,
      (&name).to_string(),
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

#[get("/user")]
pub async fn get_user(
  query: Query<GetUserQueryData>,
  data: Data<CauthConfig>
) -> impl Responder {
  // these will never error
  let mut db_conn = data.db_conn
    .acquire()
    .await
    .unwrap();

  let result = LoginSession::get_user(
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
