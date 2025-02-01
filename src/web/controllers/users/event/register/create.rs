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
        event::UserRegisterEvent,
        login_session::LoginSession
    },
    web::ServerResponse
};

#[derive(Deserialize)]
struct JsonData {
    login: String,
    password: String,
    details: Option<Value>
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
        Ok(credentials) => return ServerResponse::new(
            StatusCode::OK,
            Some(json!(credentials))
        ),
        Err(_) => return ServerResponse::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            None
        )
    };
}
