pub mod routes;

use actix_web::{
  body::BoxBody, http::{
    header::{
      HeaderName, HeaderValue
    }, StatusCode
  }, web::Data, App, HttpRequest, HttpResponse, HttpServer, Responder
};

use crate::config::CauthConfig;

pub async fn run_server(config: CauthConfig) -> std::io::Result<()> {
  simple_logger::SimpleLogger::new().init().ok();

  let binding = config.clone();
  HttpServer::new(move || {
    App::new()
      .app_data(Data::new(binding.clone()))
      .service(routes::permissions::get_permissions)
      .service(routes::permissions::post_permission)
      .service(routes::permissions::delete_permission)
      .service(routes::groups::get_groups)
      .service(routes::groups::post_group)
      .service(routes::groups::delete_group)
      .service(routes::groups::grant_permission)
      .service(routes::groups::revoke_permission)
      .service(routes::user::post_users)
      .service(routes::user::delete_users)
      .service(routes::user::get_user)
      .service(routes::user::get_user_permissions)
      .service(routes::user::post_user)
      .service(routes::user::delete_user)
      .service(routes::user::grant_group)
      .service(routes::user::revoke_group)
      .service(routes::event::commit_event)
      .service(routes::event::cancel_event)
  })
  .bind(("127.0.0.1", config.port))?
  .run()
  .await?;

  return Ok(());
}

pub struct ServerResponse {
  status: StatusCode,
  body: Option<serde_json::Value>
}

impl ServerResponse {
  pub fn new(status: StatusCode, body: Option<serde_json::Value>) -> Self {
    return Self {
      status,
      body
    };
  }
}

impl Responder for ServerResponse {
  type Body = BoxBody;

  fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
    let mut response = HttpResponse::new(self.status);
    response.headers_mut()
      .insert(
        HeaderName::from_static("content-type"), 
        HeaderValue::from_static("application/json")
      );

    if let Some(body) = &self.body {
      let body = serde_json::to_string(body).unwrap();

      return response
        .set_body(BoxBody::new(body));
    } else {
      return response;
    }
  }
}
