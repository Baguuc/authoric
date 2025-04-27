use actix_web::{
    post,
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
        group::{
            Group,
            GroupGrantError
        },
        login_session::LoginSession
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
            "details": "A group with this name do not exist"
        }))
    );
}

fn permission_not_found_error() -> ServerResponse {
    return ServerResponse::new(
        StatusCode::BAD_REQUEST,
        Some(json!({
            "code": "PERMISSION_NOT_FOUND",
            "details": "A permission with this name do not exist"
        }))
    );
}

#[post("/groups/{name}/{permission_name}")]
pub async fn controller(
    query: Query<QueryData>,
    data: Data<CauthConfig>,
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
        "authoric:group:update"
    )
    .await;

    if !permitted {
        return ServerResponse::new(
            StatusCode::UNAUTHORIZED,
            None
        );
    }

    let (group_name, permission_name) = path.into_inner();

    let result = Group::grant_permission(
        &mut db_conn,
        &group_name,
        &permission_name
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
            GroupGrantError::NotFound => return not_found_error(),
            GroupGrantError::PermissionNotFound => return permission_not_found_error()
        }
    };
}
