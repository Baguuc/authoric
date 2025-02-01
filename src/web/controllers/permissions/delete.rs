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
use crate::{
    config::CauthConfig,
    models::{
        permission::{
            Permission,
            PermissionDeleteError
        },
        login_session::LoginSession
    },
    web::ServerResponse
};

#[derive(Deserialize)]
struct QueryData {
    session_token: String,
}

type PathData = String;

fn ok() -> ServerResponse {
    return ServerResponse::new(
        StatusCode::OK,
        None
    );
}

fn not_found() -> ServerResponse {
    return ServerResponse::new(
        StatusCode::OK,
        None
    );
}

#[delete("/permissions/{name}")]
pub async fn controller(
  query: Query<QueryData>,
  data: Data<CauthConfig>,
  name: Path<PathData>
) -> impl Responder {
    // these will never error
    let mut db_conn = data.db_conn
        .acquire()
        .await
        .unwrap();

    let permitted = LoginSession::has_permission(
        &mut db_conn,
        &query.session_token,
        "cauth:permissions:delete"
    )
    .await;

    if !permitted {
        return ServerResponse::new(
            StatusCode::UNAUTHORIZED,
            None
        );
    }

    let result = Permission::delete(
        &mut db_conn,
        &name.into_inner()
    )
    .await;

    match result {
        Ok(_) => return ok(),
        Err(error) => match error {
            PermissionDeleteError::NotFound => not_found()
        }
    }
}
