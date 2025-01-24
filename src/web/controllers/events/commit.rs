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
struct QueryData {
  session_token: String
}

type PathData = i32;

#[post("/events/{id}")]
pub async fn controller(
    query: Query<QueryData>,
    path: Path<PathData>,
    data: Data<CauthConfig>
) -> impl Responder {
    // these will never error
    let mut db_conn = data.db_conn
        .begin()
        .await
        .unwrap();
    
    let id = path.into_inner();

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

    let event = match Event::retrieve(&mut db_conn, id).await {
        Ok(event) => event,
        Err(err) => match err {
            _ => return ServerResponse::new(
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

