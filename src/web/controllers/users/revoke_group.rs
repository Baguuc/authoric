use actix_web::{
    delete,
    Responder,
    http::StatusCode, 
    web::{
        Query,
        Data,
        Path
    }
};
use serde::Deserialize;
use serde_json::json;
use crate::{
    config::CauthConfig,
    models::{
        login_session::LoginSession, user::{User, UserRevokeError}
    },
    web::ServerResponse
};

#[derive(Deserialize)]
struct QueryData {
    session_token: String
}

type PathData = (String, String);

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
            "code": "NOT_FOUND",
            "details": "User with specified login do not exist"
        }))
    );
}

fn group_not_found_error() -> ServerResponse {
    return ServerResponse::new(
        StatusCode::BAD_REQUEST,
        Some(json!({
            "code": "GROUP_NOT_FOUND",
            "details": "Group with specified name do not exist"
        }))
    );
}

fn not_granted_error() -> ServerResponse {
    return ServerResponse::new(
        StatusCode::BAD_REQUEST,
        Some(json!({
            "code": "NOT_GRANTED",
            "details": "Provided user never had this group"
        }))
    );
}

#[delete("/users/{name}/{permission_name}")]
pub async fn controller(
    data: Data<CauthConfig>,
    query: Query<QueryData>,
    path: Path<PathData>
) -> impl Responder {
    // these will never error
    let mut db_conn = data.db_conn
        .begin()
        .await
        .unwrap();

    let permitted = LoginSession::has_permission(
        &mut db_conn,
        &query.session_token,
        "authoric:users:update"
    )
    .await;

    if !permitted {
        return ServerResponse::new(
            StatusCode::UNAUTHORIZED,
            None
        );
    }

    let (user_login, group_name) = path.into_inner();

    let result = User::revoke_group(
        &mut db_conn,
        &user_login,
        &group_name
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
            UserRevokeError::NotFound => return not_found_error(),
            UserRevokeError::GroupNotFound => return group_not_found_error(),
            UserRevokeError::NotGranted => return not_granted_error()
        }
    }
}
