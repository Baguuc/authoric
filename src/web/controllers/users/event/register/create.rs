use actix_web::{
    post,
    Responder,
    http::StatusCode, 
    web::{
        Json,
        Data,
        Query
    }
};
use serde::Deserialize;
use serde_json::{
    json,
    Value
};
use crate::{
    config::CauthConfig,
    models::{
        event::{user_register::UserRegisterEventInsertError, EventCredentials, UserRegisterEvent}, login_session::LoginSession, user::User
    },
    web::ServerResponse
};

#[derive(Deserialize)]
struct JsonData {
    login: String,
    password: String,
    details: Option<Value>
}

fn ok(credentials: EventCredentials) -> ServerResponse {
    return ServerResponse::new(
        StatusCode::OK,
        Some(json!(credentials))
    );
}

fn already_exist_error() -> ServerResponse {
    return ServerResponse::new(
        StatusCode::BAD_REQUEST,
        Some(json!({
            "code": "ALREADY_EXIST_ERROR",
            "details": "A user with this login do not exist"
        }))
    );
}

fn cannot_hash_error(details: String) -> ServerResponse {
    return ServerResponse::new(
        StatusCode::UNAUTHORIZED,
        Some(json!({
            "code": "UNAUTHORIZED",
            "details": format!("Couldn't hash user password: {}", details)
        }))
    );
}

#[post("/events/users/register")]
pub async fn controller(
    json: Json<JsonData>,
    data: Data<CauthConfig>
) -> impl Responder {
    // these will never error
    let mut db_conn = data.db_conn
        .acquire()
        .await
        .unwrap();
    
    let details = json.details
        .clone()
        .unwrap_or(json!({}));
    
    let result = UserRegisterEvent::insert(
        &mut db_conn,
        &json.login,
        &json.password,
        &details
    )
    .await;

    match result {
        Ok(credentials) => return ok(credentials),
        Err(error) => match error {
            UserRegisterEventInsertError::AlreadyExists => return already_exist_error(),
            UserRegisterEventInsertError::CannotHash(details) => return cannot_hash_error(details)
        }
    };
}
