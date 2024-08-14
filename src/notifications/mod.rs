mod payload;
mod utils;
mod client_manager;

use std::error::Error;

use client_manager::get_client;
use fcm_v1::message::{ FcmOptions, Message, Notification };

use self::payload::NotificationPayload;

pub use payload::{ NotificationDetails, Topic };
pub use utils::PeriodList;
pub use client_manager::setup_client;



#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotificationType {
    DayStartFullyAbsent,
    DayStartPartiallyAbsent { periods: PeriodList },

    UpdateTeacherFullyAbsent,
    UpdateTeacherPartiallyAbsent { periods: PeriodList },
    UpdateTeacherPresent,

    ReminderTeacherBackIn,
}

impl NotificationType {
    pub fn analytics_label(&self) -> &'static str {
        match self {
            Self::DayStartFullyAbsent => "day_start_fully_absent",
            Self::DayStartPartiallyAbsent { .. } => "day_start_partially_absent",
            Self::UpdateTeacherFullyAbsent => "update_teacher_fully_absent",
            Self::UpdateTeacherPartiallyAbsent { .. } => "update_teacher_partially_absent",
            Self::UpdateTeacherPresent => "update_teacher_present",
            Self::ReminderTeacherBackIn => "reminder_teacher_back_in",
        }
    }
}

pub async fn notify(payload: &NotificationPayload) -> Result<(), Box<dyn Error + Send>> {
    // Get the global Firebase client
    let client = match get_client() {
        Ok(c) => c,
        Err(e) => return Err(Box::new(e)),
    };
    
    println!("Building notification...");
    let NotificationPayload { topic, title, body, analytics } = payload;
    let notification = Notification {
        title: Some(title.clone()),
        body: body.clone(),
        image: None,
    };

    println!("Building FCM message...");
    let message = Message {
        name: Some(title.clone()),
        notification: Some(notification),
        fcm_options: Some(FcmOptions { analytics_label: analytics.clone() }),
        token: Some(topic.to_string()),
        ..Default::default()
    };

    println!("Sending notif with topic {topic:?}");
    match client.send(&message).await {
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
