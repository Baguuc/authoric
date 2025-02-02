use actix_web::{
    post,
    Responder,
    http::StatusCode, 
    web::{
        Json,
        Data,
    }
};
use serde::Deserialize;
use serde_json::json;
use crate::{
    config::CauthConfig,
    models::{
        user::{
            User,
            UserLoginError
        },
    },
    web::ServerResponse
};

#[derive(Deserialize)]
pub struct JsonData {
    login: String,
    password: String,
}

fn ok(token: String) -> ServerResponse {
    return ServerResponse::new(
        StatusCode::OK,
        Some(json!({
            "token": token
        }))
    );
}

fn not_found_error() -> ServerResponse {
    return ServerResponse::new(
        StatusCode::BAD_REQUEST,
        Some(json!({
            "details": "The user with specified login do not exist"
        }))
    );
}

fn invalid_credentials_error() -> ServerResponse {
    return ServerResponse::new(
        StatusCode::BAD_REQUEST,
        Some(json!({
            "details": "Provided credentials are invalid"
        }))
    );
}

fn cannot_hash_error(details: String) -> ServerResponse {
    return ServerResponse::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        Some(json!({
            "details": format!("Cannot create the token hash: {}", details)
        }))
    )
}

#[post("/user")]
pub async fn controller(
    json: Json<JsonData>,
    data: Data<CauthConfig>
) -> impl Responder {
    // these will never error
    let mut db_conn = data.db_conn
        .begin()
        .await
        .unwrap();

    let result = User::login(
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
        Ok(token) => return ok(token),
        Err(error) => match error {
            UserLoginError::InvalidCredentials => return invalid_credentials_error(),
            UserLoginError::NotFound => return not_found_error(),
            UserLoginError::CannotHash(details) => return cannot_hash_error(details)
        }
    };
}
