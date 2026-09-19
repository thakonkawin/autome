use actix_cors::Cors;
use actix_web::{App, HttpResponse, HttpServer, Responder, get, post, web};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use std::env;

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
struct WebhookRequest {
    destination: String,
    events: Vec<Event>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Event {
    #[serde(rename = "type")]
    event_type: String,
    message: Option<Message>,
    timestamp: u64,
    source: Source,
    #[serde(rename = "replyToken")]
    reply_token: String,
    mode: String,
    #[serde(rename = "webhookEventId")]
    webhook_event_id: String,
    #[serde(rename = "deliveryContext")]
    delivery_context: DeliveryContext,
}

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    #[serde(rename = "type")]
    message_type: String,
    id: String,
    text: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Source {
    #[serde(rename = "type")]
    source_type: String,
    #[serde(rename = "userId")]
    user_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct DeliveryContext {
    #[serde(rename = "isRedelivery")]
    is_redelivery: bool,
}

#[post("/api/line-webhook")]
async fn line_webhook(payload: web::Json<WebhookRequest>) -> impl Responder {
    println!("================ LINE WEBHOOK ================");
    println!("Destination: {}", payload.destination);

    for event in &payload.events {
        println!("Event Type: {}", event.event_type);
        println!("Timestamp: {}", event.timestamp);
        println!("Reply Token: {}", event.reply_token);
        println!("Mode: {}", event.mode);
        println!("Webhook Event ID: {}", event.webhook_event_id);

        println!("User ID: {}", event.source.user_id);

        if let Some(message) = &event.message {
            println!("Message Type: {}", message.message_type);
            println!("Message ID: {}", message.id);
            println!("Message Text: {}", message.text);
        }

        println!("Redelivery: {}", event.delivery_context.is_redelivery);

        // ส่ง Reply กลับ LINE
        if event.event_type == "message" {
            if let Err(error) = reply_webhook(&event.reply_token).await {
                eprintln!("LINE Reply Error: {}", error);
            }
        }

        println!("==============================================");
    }

    HttpResponse::Ok().finish()
}

// =========================
// LINE Reply Payload
// =========================

#[derive(Debug, Serialize)]
struct ReplyRequest {
    #[serde(rename = "replyToken")]
    reply_token: String,

    messages: Vec<ReplyMessage>,
}

#[derive(Debug, Serialize)]
struct ReplyMessage {
    #[serde(rename = "type")]
    message_type: String,

    text: String,
}

// =========================
// Reply Function
// =========================

async fn reply_webhook(reply_token: &str) -> Result<(), Box<dyn std::error::Error>> {
    let channel_access_token = env::var("CHANNEL_ACCESS_TOKEN")?;

    let url = "https://api.line.me/v2/bot/message/reply";

    let body = ReplyRequest {
        reply_token: reply_token.to_string(),
        messages: vec![
            ReplyMessage {
                message_type: "text".to_string(),
                text: "Hello, user".to_string(),
            },
            ReplyMessage {
                message_type: "text".to_string(),
                text: "May I help you?".to_string(),
            },
        ],
    };

    let client = reqwest::Client::new();

    let response = client
        .post(url)
        .bearer_auth(channel_access_token)
        .json(&body)
        .send()
        .await?;

    println!("LINE Reply Status: {}", response.status());

    let response_body = response.text().await?;

    println!("LINE Reply Response: {}", response_body);

    Ok(())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    HttpServer::new(|| {
        App::new()
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header(),
            )
            .service(health_check)
            .service(line_webhook)
    })
    .bind(("0.0.0.0", 10000))?
    .workers(2)
    .run()
    .await
}
