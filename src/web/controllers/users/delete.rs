use actix_web::{
    delete,
    Responder,
    http::StatusCode, 
    web::{
        Path,
        Data,
        Query
    }
};
use serde::Deserialize;
use serde_json::json;
use crate::{
    config::CauthConfig,
    models::{
        user::{
            User,
            UserDeleteError
        },
        event::UserDeleteEvent,
        login_session::LoginSession
    },
    web::ServerResponse
};

#[derive(Deserialize)]
pub struct QueryData {
    session_token: String
}

type PathData = String;

fn ok() -> ServerResponse {
    return ServerResponse::new(
        StatusCode::OK,
        None
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

#[delete("/users/{login}")]
pub async fn controller(
    query: Query<QueryData>,
    path: Path<PathData>,
    data: Data<CauthConfig>
) -> impl Responder {
    // these will never error
    let mut db_conn = data.db_conn
        .begin()
        .await
        .unwrap();

    let login = path.into_inner();

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
    let has_same_username = logged_user.unwrap().user_login == login;

    let permitted = has_permission || has_same_username;

    if !permitted {
        return ServerResponse::new(
            StatusCode::UNAUTHORIZED,
            None
        );
    }
    
    let result = User::delete(
        &mut db_conn,
        login
    )
    .await;
    
    match db_conn.commit().await {
        Ok(_) => (),
        Err(err) => {
            eprintln!("Error committing changes: {}", err);
        }
    };

    match result {
        Ok(_) => return ok(),
        Err(error) => match error {
            UserDeleteError::NotFound => return not_found_error()
        }
    };
}
