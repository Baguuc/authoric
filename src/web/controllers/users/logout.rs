use actix_web::{
    delete,
    Responder,
    http::StatusCode, 
    web::{
        Path,
        Data,
        Query
    }
};
use serde::Deserialize;
use serde_json::json;
use crate::{
    config::CauthConfig,
    models::{
        login_session::{LoginSession, LoginSessionDeleteError}, user::User
    },
    web::ServerResponse
};

#[derive(Deserialize)]
pub struct QueryData {
    session_token: String
}

fn ok() -> ServerResponse {
    return ServerResponse::new(
        StatusCode::OK,
        None
    );
}

fn not_found_error() -> ServerResponse {
    return ServerResponse::new(
        StatusCode::BAD_REQUEST,
        Some(json!({
            "code": "NOT_FOUND",
            "details": "Session associated with this token do not exist"
        }))
    );
}

#[delete("/user")]
pub async fn controller(
    query: Query<QueryData>,
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
        Ok(_) => return ok(),
        Err(error) => match error {
            LoginSessionDeleteError::NotFound => return not_found_error()
        } 
    };
}

