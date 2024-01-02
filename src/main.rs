use status_loop::status_loop;

mod env;
mod gql;
mod notifications;
mod status_loop;

use actix_web::{get, App, HttpResponse, HttpServer, Responder};

#[get("/")]
async fn ping() -> impl Responder {
    HttpResponse::Ok()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env::ensure_env();

    tokio::spawn(async {
        status_loop().await.unwrap();
    });

    return HttpServer::new(|| {
        App::new()
            .service(ping)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await;
}

