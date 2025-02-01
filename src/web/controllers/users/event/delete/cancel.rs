use actix_web::{
    post,
    Responder,
    http::StatusCode, 
    web::{
        Json,
        Data
    }
};
use serde::Deserialize;
use serde_json::json;
use crate::{
    config::CauthConfig,
    models::{
        user::User,
        event::UserDeleteEvent,
        login_session::LoginSession
    },
    web::ServerResponse
};

#[derive(Deserialize)]
struct JsonData {
    id: i32,
    key: String
}

#[post("/events/users/delete/cancel")]
pub async fn controller(
    json: Json<JsonData>,
    data: Data<CauthConfig>
) -> impl Responder {
    // these will never error
    let mut db_conn = data.db_conn
        .acquire()
        .await
        .unwrap();

    let result = UserDeleteEvent::cancel(
        &mut db_conn,
        &json.id,
        &json.key
    )
    .await;

    match result {
        Ok(_) => return ServerResponse::new(
            StatusCode::OK,
            None
        ),
        Err(_) => return ServerResponse::new(
            StatusCode::UNAUTHORIZED,
            None
        )
    }
}
