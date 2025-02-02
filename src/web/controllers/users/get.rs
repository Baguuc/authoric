use actix_web::{
    Responder,
    get,
    http::StatusCode,
    web::{
        Query,
        Data
    }
};
use serde::Deserialize;
use serde_json::json;
use crate::{
    config::CauthConfig,
    models::{
        user::User,
        login_session::{
            LoginSession,
            LoginSessionGetUserError
        }
    },
    web::ServerResponse
};

#[derive(Deserialize)]
struct QueryData {
  session_token: String
}

fn ok(user: User) -> ServerResponse {
    return ServerResponse::new(
        StatusCode::OK,
        Some(json!(user))
    );
}

fn not_found_error() -> ServerResponse {
    return ServerResponse::new(
        StatusCode::BAD_REQUEST,
        Some(json!({
            "code": "NOT_FOUND",
            "details": "The session associated with this token was not found"
        }))
    );
}

#[get("/user")]
pub async fn controller(
  query: Query<QueryData>,
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
        Ok(user) => return ok(user),
        Err(error) => match error {
            LoginSessionGetUserError::NotFound => return not_found_error() 
        }
    };
}
