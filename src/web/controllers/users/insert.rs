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
use serde_json::{
    json,
    Value
};
use crate::{
    config::CauthConfig,
    models::{
        user::{
            User,
            UserInsertError
        },
    },
    web::ServerResponse
};

#[derive(Deserialize)]
struct JsonData {
    login: String,
    password: String,
    details: Option<Value>
}

fn ok() -> ServerResponse {
    return ServerResponse::new(
        StatusCode::OK,
        None
    );
}

fn name_error() -> ServerResponse {
    return ServerResponse::new(
        StatusCode::BAD_REQUEST,
        Some(json!({
            "code": "NAME_ERROR",
            "details": "A user with this login already exists"
        }))
    );
}

fn cannot_hash_error(details: String) -> ServerResponse {
    return ServerResponse::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        Some(json!({
            "code": "CANNOT_HASH",
            "details": format!("Cannot hash the user's password: {}", details)
        }))
    );
}

#[post("/users")]
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
    
    let result = User::insert(
        &mut db_conn, 
        &json.login, 
        &json.password, 
        &details
    )
    .await;
    
    match result {
        Ok(_) => return ok(),
        Err(error) => match error {
            UserInsertError::NameError => return name_error(),
            UserInsertError::CannotHash(details) => return cannot_hash_error(details)
        }
    };
}
