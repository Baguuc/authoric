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
        event::{user_login::UserLoginEventInsertError, EventCredentials, UserLoginEvent}, login_session::{
            LoginSession
        }, user::User
    },
    web::ServerResponse
};

#[derive(Deserialize)]
pub struct JsonData {
    login: String,
    password: String,
}

fn ok(credentials: EventCredentials) -> ServerResponse {
    return ServerResponse::new(
        StatusCode::OK,
        Some(json!(credentials))
    );
}

fn user_not_found_error() -> ServerResponse {
    return ServerResponse::new(
        StatusCode::BAD_REQUEST,
        Some(json!({
            "details": "User with this login do not exist"
        }))
    );
}

fn unauthorized_error() -> ServerResponse {
    return ServerResponse::new(
        StatusCode::UNAUTHORIZED,
        Some(json!({
            "details": "Invalid password!"
        }))
    );
}

#[post("/events/users/login")]
pub async fn controller(
    json: Json<JsonData>,
    data: Data<CauthConfig>
) -> impl Responder {
    // these will never error
    let mut db_conn = data.db_conn
        .begin()
        .await
        .unwrap();
    
    let result = UserLoginEvent::insert(
        &mut db_conn, 
        &json.login,
        &json.password
    )
    .await;

    match db_conn.commit().await {
        Ok(_) => (),
        Err(err) => {
            eprintln!("Error committing changes: {}", err);
        }
    };
    
    match result {
        Ok(credentials) => return ok(credentials),
        Err(error) => match error {
            UserLoginEventInsertError::Unauthorized => return unauthorized_error(),
            UserLoginEventInsertError::UserNotFound => return user_not_found_error()
        }
    };
}

