use actix_web::{
    post,
    http::StatusCode,
    web::{
        Data,
        Path,
        Query
    },
    Responder
};
use serde::Deserialize;
use serde_json::json;
use crate::{
    config::CauthConfig,
    models::{
    event::EventRetrieveError,
        Event,
        LoginSession
    },
    web::ServerResponse
};

#[derive(Deserialize)]
struct QueryData {
  key: String
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
    
    let result = Event::commit(
        &mut db_conn,
        id,
        &query.key
    ).await;

    if result.is_err() {
        return ServerResponse::new(
            StatusCode::UNAUTHORIZED,
            None
        );
    }

    let response = result.unwrap();

    match db_conn.commit().await {
        Ok(_) => (),
        Err(err) => {
            eprintln!("Error committing changes: {}", err);
        }
    };

    return ServerResponse::new(
        StatusCode::OK,
        response
    );
}

