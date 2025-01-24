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
        login_session::LoginSession
    },
    web::ServerResponse
};

#[derive(Deserialize)]
struct QueryData {
  session_token: String
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
