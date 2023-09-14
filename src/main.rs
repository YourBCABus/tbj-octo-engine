use notifications::{issue_teacher_absence_notification, notification_loop};

mod notifications;
mod gql;

use actix_web::{get, App, HttpResponse, HttpServer, Responder};

#[get("/")]
async fn ping() -> impl Responder {
    HttpResponse::Ok()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {    
    match dotenvy::dotenv() {
        Ok(_) => println!("Successfully loaded .env"),
        Err(e) => eprintln!("Error loading dotenv: {}", e),
    };

    tokio::spawn(async {
        notification_loop().await.unwrap();
    });

    return HttpServer::new(|| {
        App::new()
            .service(ping)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await;
}

