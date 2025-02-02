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
        event::{user_register::UserRegisterEventCommitError, EventCredentials, UserRegisterEvent}, login_session::LoginSession, user::User
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

#[post("/events/users/register/commit")]
pub async fn controller(
    json: Json<JsonData>,
    data: Data<CauthConfig>
) -> impl Responder {
    // these will never error
    let mut db_conn = data.db_conn
        .begin()
        .await
        .unwrap();

    let result = UserRegisterEvent::commit(
        &mut db_conn,
        &json.id,
        &json.key
    )
    .await;

    match db_conn.commit().await {
        Ok(_) => (),
        Err(err) => {
            eprintln!("Error while commiting changes to the database: {}", err);
        }
    };

    match result {
        Ok(_) => return ok(),
        Err(error) => match error {
            UserRegisterEventCommitError::NotFound => return not_found_error(),
            UserRegisterEventCommitError::Unauthorized => return unauthorized_error()
        } 
    }
}
