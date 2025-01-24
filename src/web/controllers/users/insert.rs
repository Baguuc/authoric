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
        user::User,
        login_session::LoginSession
    },
    web::ServerResponse
};

#[derive(Deserialize)]
pub struct JsonData {
    login: String,
    password: String,
    details: Option<Value>
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
