use actix_cors::Cors;
use actix_web::{App, HttpRequest, HttpResponse, HttpServer, Responder, get, post, web};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use std::env;

// =========================
// Health Check
// =========================

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
        app_name: "autome-api",
        app_version: "v0.0.3",
    })
}

// =========================
// Telegram Webhook Models
// =========================

#[derive(Debug, Deserialize)]
struct TelegramUpdate {
    message: Option<TelegramMessage>,
}

#[derive(Debug, Deserialize)]
struct TelegramMessage {
    text: Option<String>,
    chat: TelegramChat,
}

#[derive(Debug, Deserialize)]
struct TelegramChat {
    id: i64,
}

// =========================
// Telegram Send Message
// =========================

#[derive(Debug, Serialize)]
struct TelegramSendMessage {
    chat_id: i64,
    text: String,
}

// =========================
// Telegram Webhook Handler
// =========================

#[post("/api/telegram-webhook")]
async fn telegram_webhook(req: HttpRequest, payload: web::Json<TelegramUpdate>) -> HttpResponse {
    println!("================ TELEGRAM WEBHOOK ================");

    // ตรวจสอบ Webhook Secret
    let expected_secret = match env::var("TELEGRAM_WEBHOOK_SECRET") {
        Ok(secret) => secret,
        Err(_) => {
            eprintln!("TELEGRAM_WEBHOOK_SECRET is not configured");
            return HttpResponse::InternalServerError().body("Webhook secret is not configured");
        }
    };

    let received_secret = req
        .headers()
        .get("X-Telegram-Bot-Api-Secret-Token")
        .and_then(|value| value.to_str().ok());

    if received_secret != Some(expected_secret.as_str()) {
        return HttpResponse::Unauthorized().finish();
    }

    let message = match &payload.message {
        Some(message) => message,
        None => {
            // Update อาจเป็น event ประเภทอื่นที่ไม่ใช่ข้อความ
            return HttpResponse::Ok().finish();
        }
    };

    let text = match &message.text {
        Some(text) => text.trim(),
        None => {
            // ไม่ใช่ข้อความตัวอักษร เช่น รูปภาพหรือสติกเกอร์
            return HttpResponse::Ok().finish();
        }
    };

    let chat_id = message.chat.id;

    println!("Chat ID: {}", chat_id);
    println!("Message: {}", text);

    let reply = match text {
        "/start" => "สวัสดี! ผมคือ autome 🤖\nยินดีต้อนรับสู่ Personal Bot ของคุณ",
        "/help" => "คำสั่งที่ใช้งานได้:\n/start - เริ่มต้นใช้งาน\n/help - ดูคำสั่ง\n/status - ตรวจสอบสถานะ",
        "/status" => "Autome API is running normally. ✅",
        _ => "Autome ได้รับข้อความของคุณแล้ว ✅",
    };

    // ส่งคำตอบกลับ Telegram
    if let Err(error) = reply_telegram(chat_id, reply).await {
        eprintln!("Telegram Reply Error: {}", error);

        // รับ Webhook สำเร็จแล้ว แต่การส่งข้อความกลับล้มเหลว
        // ตอบ 200 เพื่อไม่ให้ Telegram ส่ง Update เดิมซ้ำโดยอัตโนมัติ
        return HttpResponse::Ok().finish();
    }

    println!("===================================================");

    HttpResponse::Ok().finish()
}

// =========================
// Telegram Reply Function
// =========================

async fn reply_telegram(chat_id: i64, text: &str) -> Result<(), Box<dyn std::error::Error>> {
    let bot_token = env::var("TELEGRAM_BOT_TOKEN")?;

    let url = format!("https://api.telegram.org/bot{}/sendMessage", bot_token);

    let body = TelegramSendMessage {
        chat_id,
        text: text.to_string(),
    };

    let client = reqwest::Client::new();

    let response = client.post(&url).json(&body).send().await?;

    let status = response.status();
    let response_body = response.text().await?;

    println!("Telegram Reply Status: {}", status);

    if !status.is_success() {
        return Err(format!("Telegram API error: {}", response_body).into());
    }

    Ok(())
}

// =========================
// Main
// =========================

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| "10000".to_string())
        .parse()
        .expect("PORT must be a valid port number");

    println!("Starting autome-api on port {}", port);

    HttpServer::new(|| {
        App::new()
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header(),
            )
            .service(health_check)
            .service(telegram_webhook)
    })
    .bind(("0.0.0.0", port))?
    .workers(2)
    .run()
    .await
}
