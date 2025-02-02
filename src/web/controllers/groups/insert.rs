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
use serde_json::json;
use serde::Deserialize;
use crate::{
    config::CauthConfig,
    models::{
        group::{
            Group,
            GroupInsertError
        },
        login_session::LoginSession
    },
    web::ServerResponse
};

#[derive(Deserialize)]
struct QueryData {
    session_token: String,
}

#[derive(Deserialize)]
struct JsonData {
    name: String,
    description: String,
    permissions: Vec<String>
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
            "details": "A group with this name already exist."
        }))
    );
}

fn permission_not_found_error() -> ServerResponse {
    return ServerResponse::new(
        StatusCode::BAD_REQUEST,
        Some(json!({
            "code": "PERMISSION_NOT_FOUND",
            "details": "One of the listed permissions do not exist."
        }))
    );

}

#[post("/groups")]
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

    let permitted = LoginSession::has_permission(
        &mut db_conn,
        &query.session_token,
        "cauth:groups:post"
    )
    .await;

    if !permitted {
        return ServerResponse::new(
            StatusCode::UNAUTHORIZED,
            None
        );
    }

    let result = Group::insert(
        &mut db_conn,
        &json.name,
        &json.description,
        &json.permissions
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
            GroupInsertError::NameError => return name_error(),
            GroupInsertError::PermissionNotFound => return permission_not_found_error()
        }
    }
}
