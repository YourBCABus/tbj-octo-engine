use lazy_static::lazy_static;
use tokio::sync::OnceCell;

lazy_static! {
    pub static ref GRAPHQL_ENDPOINT_URL: String = {
        dotenvy::dotenv().unwrap();

        dotenvy::var("GRAPHQL_ENDPOINT_URL").unwrap()
    };

    pub static ref DISCORD_WEBHOOK_URL: String = {
        dotenvy::dotenv().unwrap();

        dotenvy::var("DISCORD_WEBHOOK_URL").unwrap()
    };


    pub static ref CLIENT_ID: String = {
        dotenvy::dotenv().unwrap();

        dotenvy::var("CLIENT_ID").unwrap()
    };

    pub static ref CLIENT_SECRET: String = {
        dotenvy::dotenv().unwrap();

        dotenvy::var("CLIENT_SECRET").unwrap()
    };

    static ref SERVICE_KEY_PATH: String = {
        dotenvy::dotenv().unwrap();

        dotenvy::var("SERVICE_KEY_PATH").unwrap()
    };
}

pub async fn ensure_env() {
    use std::hint::black_box;

    black_box(&GRAPHQL_ENDPOINT_URL);
    black_box(&DISCORD_WEBHOOK_URL);
    black_box(&CLIENT_ID);
    black_box(&CLIENT_SECRET);

    black_box(&SERVICE_KEY_PATH);
    black_box(authenticator().await);
}


pub async fn authenticator() -> &'static fcm_v1::auth::Authenticator {
    static SERVICE_KEY: OnceCell<fcm_v1::auth::Authenticator> = OnceCell::const_new();
    SERVICE_KEY.get_or_init(|| async {
        fcm_v1::auth::Authenticator::service_account_from_file(
            SERVICE_KEY_PATH.as_str()
        ).await.unwrap()
    }).await
}
