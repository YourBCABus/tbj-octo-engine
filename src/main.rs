use notifications::notification_loop;

mod notifications;
mod gql;
mod env;

use actix_web::{get, App, HttpResponse, HttpServer, Responder};

#[get("/")]
async fn ping() -> impl Responder {
    HttpResponse::Ok()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env::ensure_env();

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

