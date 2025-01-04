pub mod routes;

use actix_web::{
  body::BoxBody, http::{
    header::{
      HeaderName, HeaderValue
    }, StatusCode
  }, App, HttpRequest, HttpResponse, HttpServer, Responder
};

pub async fn run_server(port: u16) -> std::io::Result<()> {
  HttpServer::new(|| {
    App::new()
      .service(routes::permissions::get_permissions)
      .service(routes::permissions::post_permission)
  })
  .bind(("127.0.0.1", port))?
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
