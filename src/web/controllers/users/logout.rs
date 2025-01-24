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
        user::User,
        login_session::LoginSession
    },
    web::ServerResponse
};

#[derive(Deserialize)]
pub struct QueryData {
    session_token: String
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

