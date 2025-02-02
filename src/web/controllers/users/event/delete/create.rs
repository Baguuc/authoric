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
use serde_json::json;
use crate::{
    config::CauthConfig,
    models::{
        event::{user_delete::UserDeleteEventInsertError, EventCredentials, UserDeleteEvent}, login_session::LoginSession, user::User
    },
    web::ServerResponse
};

#[derive(Deserialize)]
struct QueryData {
    session_token: String
}

#[derive(Deserialize)]
struct JsonData {
    login: String
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
            "code": "USER_NOT_FOUND",
            "details": "User with this login do not exist"
        }))
    );
}

#[post("/events/users/delete")]
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

    let has_permission = LoginSession::has_permission(
        &mut db_conn,
        &query.session_token,
        &"cauth:users:delete".to_string()
    )
    .await;

    let logged_user = LoginSession::retrieve(
        &mut db_conn,
        &query.session_token
    ).await;

    if !has_permission && logged_user.is_err() { 
        return ServerResponse::new(
            StatusCode::UNAUTHORIZED,
            None
        );
    }
    let has_same_username = logged_user.unwrap().user_login == json.login;

    let permitted = has_permission || has_same_username;

    if !permitted {
        return ServerResponse::new(
            StatusCode::UNAUTHORIZED,
            None
        );
    }
    
    let result = UserDeleteEvent::insert(
        &mut db_conn,
        &json.login
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
            UserDeleteEventInsertError::UserNotFound => return user_not_found_error()
        }
    };
}
