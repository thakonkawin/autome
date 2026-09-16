use actix_web::{App, HttpServer, Responder, get, web};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: &'static str,
    app_name: &'static str,
    app_version: &'static str,
}

#[get("/api")]
async fn health_check() -> impl Responder {
    web::Json(HealthResponse {
        status: "ok",
        app_name: "autome-api-test",
        app_version: "v0.0.2",
    })
}

#[derive(Debug, Serialize, Deserialize)]
struct User {
    id: u64,
    name: String,
    email: String,
}

#[get("/api/users")]
async fn get_users() -> impl Responder {
    let users = vec![
        User {
            id: 1,
            name: "John".to_string(),
            email: "john@example.com".to_string(),
        },
        User {
            id: 2,
            name: "Jane".to_string(),
            email: "jane@example.com".to_string(),
        },
    ];

    web::Json(users)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(health_check).service(get_users))
        .bind(("0.0.0.0", 10000))?
        .workers(2)
        .run()
        .await
}
