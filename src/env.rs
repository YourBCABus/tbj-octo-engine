use lazy_static::lazy_static;

lazy_static! {
    pub static ref FIREBASE_API_KEY: String = {
        dotenvy::dotenv().unwrap();

        dotenvy::var("FIREBASE_API_KEY").unwrap()
    };

    pub static ref GRAPHQL_ENDPOINT_URL: String = {
        dotenvy::dotenv().unwrap();

        dotenvy::var("GRAPHQL_ENDPOINT_URL").unwrap()
    };

    pub static ref DISCORD_WEBHOOK_URL: String = {
        dotenvy::dotenv().unwrap();

        dotenvy::var("DISCORD_WEBHOOK_URL").unwrap()
    };
}

pub fn ensure_env() {
    use std::hint::black_box;

    black_box(&FIREBASE_API_KEY);
    black_box(&GRAPHQL_ENDPOINT_URL);
    black_box(&DISCORD_WEBHOOK_URL);
}
