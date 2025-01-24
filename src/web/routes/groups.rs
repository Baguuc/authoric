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
    .begin()
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

  if auto_commit {
    let result = Group::delete(
      &mut db_conn,
      &name
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
    let result = Group::event().delete(
      &mut db_conn,
      &name,
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
      Err(_) =>  return ServerResponse::new(
        StatusCode::BAD_REQUEST,
        None
      ),
    }
  }
}


#[post("/groups/{name}/{permission_name}")]
pub async fn grant_permission(
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
    "cauth:group:update"
  )
  .await;

  if !permitted {
    return ServerResponse::new(
      StatusCode::UNAUTHORIZED,
      None
    );
  }

  let (group_name, permision_name) = path.into_inner();

  let result = Group::grant_permission(
    &mut db_conn,
    &group_name,
    &permission_name
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

#[delete("/groups/{name}/{permission_name}")]
pub async fn revoke_permission(
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
    "cauth:groups:update"
  )
  .await;

  if !permitted {
    return ServerResponse::new(
      StatusCode::UNAUTHORIZED,
      None
    );
  }

  let (group_name, permision_name) = path.into_inner();

  let result = Group::revoke_permission(
    &mut db_conn,
    &group_name,
    &permission_name
  )
  .await;

  match result {
      Ok(_) => (),
      Err(_) => {
          return ServerResponse::new(
            StatusCode::BAD_REQUEST,
            None
          );
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
