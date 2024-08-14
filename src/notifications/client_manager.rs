use std::time::Duration;

use tokio::sync::OnceCell;

use crate::env::authenticator;

#[derive(Debug, Clone, Copy)]
pub struct ClientInitErr;

impl std::fmt::Display for ClientInitErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Client was not initialized yet")
    }
}
impl std::error::Error for ClientInitErr {}



static CLIENT: OnceCell<fcm_v1::Client> = OnceCell::const_new();

pub fn get_client() -> Result<&'static fcm_v1::Client, ClientInitErr> {
    CLIENT.get().ok_or(ClientInitErr)
}

pub fn init_client(
    auth: fcm_v1::auth::Authenticator,
    project: &str,
    timeout: Option<Duration>,
) -> Result<(), ClientInitErr> {
    if let Err(e) = CLIENT.set(fcm_v1::Client::new(
        auth,
        project,
        cfg!(debug_assertions),
        timeout.unwrap_or(Duration::MAX)
    )) {
        eprintln!("Failed to initialize FireBase client.");

        match e {
            tokio::sync::SetError::InitializingError(_) => {
                println!("Client is being initialized from another place.");
            },
            tokio::sync::SetError::AlreadyInitializedError(_) => {
                println!("Client was already initialized.");
            },
        };

        Err(ClientInitErr)
    } else {
        Ok(())
    }
}

pub async fn setup_client(project: &str) {
    init_client(
        authenticator().await.clone(),
        project,
        Some(std::time::Duration::from_millis(5000)),
    ).unwrap();
}
