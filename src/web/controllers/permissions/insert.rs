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
        permission::Permission,
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
    description: String
}

#[post("/permissions")]
pub async fn controller(
    query: Query<QueryData>,
    json: Json<JsonData>,
    data: Data<CauthConfig>
) -> impl Responder {
    // these will never error
    let mut db_conn = data.db_conn
        .acquire()
        .await
        .unwrap();

    let permitted = LoginSession::has_permission(
        &mut db_conn,
        &query.session_token,
        "cauth:permissions:post"
    )
    .await;

    if !permitted {
        return ServerResponse::new(
          StatusCode::UNAUTHORIZED,
          None
        );
    }

    let result = Permission::insert(
    &mut db_conn,
    &json.name,
    &json.description
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
    }
}
