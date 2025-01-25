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
        login_session::{
            LoginSession,
            LoginSessionStatus
        }
    },
    web::ServerResponse
};

#[derive(Deserialize)]
pub struct QueryData {
    auto_commit: Option<bool>
}

#[derive(Deserialize)]
pub struct JsonData {
    login: String,
    password: String,
}

#[post("/user")]
pub async fn controller(
    query: Query<QueryData>,
    json: Json<JsonData>,
    data: Data<CauthConfig>
) -> impl Responder {
    // these will never error
    let mut db_conn = data.db_conn
        .begin()
        .await
        .unwrap();

    let auto_commit = query.auto_commit
        .unwrap_or(true);

    if auto_commit {
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
    } else {
        let result = User::event().login(
            &mut db_conn, 
            &json.login,
            &json.password
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
}

