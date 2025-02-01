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
        event::UserRegisterEvent,
        login_session::LoginSession
    },
    web::ServerResponse
};

#[derive(Deserialize)]
struct JsonData {
    id: i32,
    key: String
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
