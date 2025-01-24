use actix_web::{
    Responder,
    get,
    http::StatusCode,
    web::{
        Path,
        Query,
        Data
    }
};
use serde::Deserialize;
use serde_json::json;
use crate::{
    config::CauthConfig,
    models::{
        user::User,
        login_session::LoginSession
    },
    web::ServerResponse
};

#[derive(Deserialize)]
struct QueryData {
    session_token: String
}

type PathData = String;

#[get("/user/permissions/{permission_name}")]
pub async fn controller(
    path: Path<PathData>,
    query: Query<QueryData>,
    data: Data<CauthConfig>
) -> impl Responder {
    // these will never rror
    let mut db_conn = data.db_conn
        .acquire()
        .await
        .unwrap();

    let permission_name = path.into_inner();

    let result = LoginSession::has_permission(
        &mut db_conn,
        &query.session_token,
        &permission_name
    )
    .await;

    return ServerResponse::new(
        StatusCode::OK,
        Some(json!({
            "has": result
        }))
    );
}

