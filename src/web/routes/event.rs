use actix_web::{
  delete, http::StatusCode, post, web::{
    Data, Path, Query
  }, Responder
};
use serde::Deserialize;
use serde_json::json;
use crate::{
  config::CauthConfig, models::{
    event::EventRetrieveError, Event, LoginSession
  }, web::ServerResponse
};

#[derive(Deserialize)]
struct CommitEventQueryData {
  session_token: String
}

#[post("/events/{id}")]
pub async fn commit_event(
  query: Query<CommitEventQueryData>,
  data: Data<CauthConfig>,
  id: Path<i64>
) -> impl Responder {
  // these will never error
  let mut db_conn = data.db_conn
    .acquire()
    .await
    .unwrap();

  let permitted = LoginSession::has_permission(
    &mut db_conn,
    &query.session_token,
    format!("events:use:{}", id).as_str()
  )
  .await;

  if !permitted {
    return ServerResponse::new(
      StatusCode::UNAUTHORIZED,
      None
    );
  }

  let event = match Event::retrieve(&mut db_conn, *id).await {
    Ok(event) => event,
    Err(err) => match err {
      EventRetrieveError::NotFound => return ServerResponse::new(
        StatusCode::BAD_REQUEST, 
        None
      )
    }
  };

  match event.commit(&mut db_conn).await {
    Ok(_) => (),
    Err(err) => return ServerResponse::new(
      StatusCode::BAD_REQUEST,
      Some(json!({
        "details": err.to_string()
      }))
    )
  };

  return ServerResponse::new(
    StatusCode::OK,
    None
  );
}

#[derive(Deserialize)]
struct CancelEventQueryData {
  session_token: String
}

#[delete("/events/{id}")]
pub async fn cancel_event(
  query: Query<CancelEventQueryData>,
  data: Data<CauthConfig>,
  id: Path<i64>
) -> impl Responder {
  // these will never error
  let mut db_conn = data.db_conn
    .acquire()
    .await
    .unwrap();

  let permitted = LoginSession::has_permission(
    &mut db_conn,
    &query.session_token,
    format!("events:use:{}", id).as_str()
  )
  .await;

  if !permitted {
    return ServerResponse::new(
      StatusCode::UNAUTHORIZED,
      None
    );
  }

  let event = match Event::retrieve(&mut db_conn, *id).await {
    Ok(event) => event,
    Err(err) => match err {
      EventRetrieveError::NotFound => return ServerResponse::new(
        StatusCode::BAD_REQUEST, 
        None
      )
    }
  };

  let _ = event.delete(&mut db_conn).await;

  return ServerResponse::new(
    StatusCode::OK,
    None
  );
}