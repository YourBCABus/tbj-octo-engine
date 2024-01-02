mod payload;

use std::error::Error;

use fcm::{Client, MessageBuilder, NotificationBuilder};

use self::payload::NotificationPayload;

pub use payload::{ NotificationDetails, Topic };



#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotificationType {
    DayStartFullyAbsent,
    DayStartPartiallyAbsent { periods: Vec<String> },

    UpdateTeacherFullyAbsent,
    UpdateTeacherPartiallyAbsent { periods: Vec<String> },
    UpdateTeacherPresent,

    ReminderTeacherBackIn,
}

pub async fn notify(payload: &NotificationPayload) -> Result<(), Box<dyn Error + Send>> {
    // Pull API key loaded at startup
    let api_key = &crate::env::FIREBASE_API_KEY;

    // Create new Firebase client
    // NOTE: Maybe put this into a global client to reduce load/API calls. (Not
    // sure if this would even do that.)   
    let client = Client::new();
    
    let NotificationPayload { topic, title, body } = payload;
    let topic = topic.to_string();

    let mut builder = NotificationBuilder::new();

    builder.title(title);
    if let Some(body) = body {
        builder.body(body);
    }

    let notification = builder.finalize();

    println!("Sending notif with topic {topic:?}");
    let mut builder = MessageBuilder::new(api_key.as_str(), &topic);
    builder.notification(notification);
    let message = builder.finalize();

    match client.send(message).await {
        Ok(response) => {
            println!("Sent: {:?}", response);
            Ok(())
        },
        Err(e) => {
            eprintln!("[ERROR] Error sending notification: {}", e);
            // TODO - move webhook url to lazy statics make webhook request to
            // send webhook request using reqwest to discord webhook without
            // serde json
            let webhook_url = crate::env::DISCORD_WEBHOOK_URL.to_string();
            let client = reqwest::Client::new();
            
            const ERROR_NOTIF_PAYLOAD: &str = r##" { "content": "Failed to send Firebase notification. ☹️☹️" } "##;
            
            let result = client.post(webhook_url)
                .header("Content-Type", "application/json")
                .body(ERROR_NOTIF_PAYLOAD)
                .send()
                .await;
            
            match result {
                Ok(res) => {
                    println!("[ERROR] Successfully sent error notification (this might not actually have done anything, check status code): {:?}", res);
                    Err(Box::new(e))
                },
                Err(e) => {
                    eprintln!("[ERROR] Error sending error notification: {}", e);
                    Err(Box::new(e))
                },
            }
            

        }
    }
}

pub async fn issue_notif(details: NotificationDetails) -> Result<(), Box<dyn Error + Send>> {
    let payload = details.build();
    notify(&payload).await
}
