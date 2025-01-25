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
        user::User,
        login_session::LoginSession
    },
    web::ServerResponse
};

#[derive(Deserialize)]
pub struct QueryData {
    session_token: String,
    auto_commit: bool
}

type PathData = String;

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

    let permitted = LoginSession::has_permission(
        &mut db_conn,
        &query.session_token,
        &format!("users:delete:{}", login)
    )
    .await;

    if !permitted {
        return ServerResponse::new(
            StatusCode::UNAUTHORIZED,
            None
        );
    }

    if query.auto_commit {
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
            Ok(_) => return ServerResponse::new(
                StatusCode::OK,
                None
            ),
            Err(_) => return ServerResponse::new(
                StatusCode::BAD_REQUEST,
                None
            )
        }
    } else { 
        let result = User::event().delete(
            &mut db_conn, 
            &login
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
