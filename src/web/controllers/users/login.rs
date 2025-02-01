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
        event::UserLoginEvent,
        login_session::{
            LoginSession,
            LoginSessionStatus
        }
    },
    web::ServerResponse
};

#[derive(Deserialize)]
pub struct JsonData {
    login: String,
    password: String,
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
        &json.password,
        LoginSessionStatus::Commited
    )
    .await;

    match db_conn.commit().await {
        Ok(_) => (),
        Err(err) => {
            eprintln!("Error committing changes: {}", err);
        }
    };

    match result {
        Ok(token) => return ServerResponse::new(
            StatusCode::OK,
            Some(json!({
            "token": token
            }))
        ),
        Err(_) => return ServerResponse::new(
            StatusCode::BAD_REQUEST,
            None
        )
    };
}

