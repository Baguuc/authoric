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
        event::user_delete::{
            UserDeleteEvent,
            UserDeleteEventCancelError
        },
        login_session::LoginSession
    },
    web::ServerResponse
};

#[derive(Deserialize)]
struct JsonData {
    id: i32,
    key: String
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
            "details": "Event with this id do not exist"
        }))
    );
}

fn unauthorized_error() -> ServerResponse {
    return ServerResponse::new(
        StatusCode::UNAUTHORIZED,
        Some(json!({
            "details": "You are not authorized to do that!"
        }))
    );
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
        Ok(_) => return ok(),
        Err(error) => match error {
            UserDeleteEventCancelError::NotFound => return not_found_error(),
            UserDeleteEventCancelError::Unauthorized => return unauthorized_error()
        }
    }
}
