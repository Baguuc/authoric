use actix_web::{
    Responder,
    get,
    http::StatusCode,
    web::{
        Query,
        Data
    }
};
use serde::Deserialize;
use serde_json::json;
use crate::{
    config::CauthConfig,
    models::{
        Order,
        group::Group,
        login_session::LoginSession
    },
    web::ServerResponse
};

#[derive(Deserialize)]
struct QueryData {
    session_token: String,
    order_in: Option<Order>,
    page: Option<usize>
}

#[get("/groups")]
pub async fn controller(
    query: Query<QueryData>,
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
        "authoric:groups:get"
    )
    .await;

    if !permitted {
        return ServerResponse::new(
            StatusCode::UNAUTHORIZED,
            None
        );
    }
      
    let result = Group::list(
        &mut db_conn,
        query.order_in,
        Some(query.page.unwrap_or(0) * 10),
        Some(10)
    )
    .await
    .unwrap();
    
    return ServerResponse::new(
        StatusCode::OK,
        Some(json!(result))
    );
}
