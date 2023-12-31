mod payload;

use std::error::Error;

use chrono::Timelike;
use fcm::{Client, MessageBuilder, NotificationBuilder};

use crate::gql::{fetch_teacher_periods, period_teacher::{self, PeriodTeacherPeriods}};

/// The number of seconds ahead of time to look when determining the
/// upcoming/current period
const PERIOD_ADVANCE: f64 = 300.0;




#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationType<'a> {
    DayStartFullyAbsent,
    DayStartPartiallyAbsent { periods: &'a [&'a str] },

    UpdateTeacherFullyAbsent,
    UpdateTeacherPartiallyAbsent { periods: &'a [&'a str] },
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
    
    let body = format!("{} is absent this period. Sign into the upper cafeteria.", payload.teacher_name);
    let title = format!("{} is absent", payload.teacher_name);

    let mut builder = NotificationBuilder::new();
    builder.title(&title);
    builder.body(&body);

    let notification = builder.finalize();

    let mut builder = MessageBuilder::new(api_key.as_str(), &payload.topic);
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

pub async fn issue_teacher_absence_notification(current_period: &PeriodTeacherPeriods) -> Result<(), Box<dyn Error + Send>> {
    let mut notification_targets: Vec<NotificationPayload> = Vec::new();

    println!("Current period: {:#?}", current_period);

    let current_notification_targets: Vec<NotificationPayload> = current_period.teachers_absent.iter().map(|absent_teacher| {
        NotificationPayload {
            topic: format!("/topics/{}.{}", current_period.id, absent_teacher.id),
            teacher_name: absent_teacher.name.normal.to_string(),
            body: None,
        }
    }).collect();

    notification_targets.extend(current_notification_targets);
    println!("Notification targets: {:#?}", notification_targets);          

    futures::future::join_all(
        notification_targets.iter().map(|target| {
            println!("Target: {:#?}", target);
            notify(target)
        })
    ).await.into_iter().collect()
}

pub async fn notification_loop() -> Result<(), Box<dyn Error + Send>> {
    let mut prior_period = get_current_period().await;

    if let Some(cur) = prior_period.clone() {
        match issue_teacher_absence_notification(&cur).await {
            Ok(_) => println!("[STARTUP NOTIF] Successfully issued teacher absence notification for period {:#?}", &cur),
            Err(e) => eprintln!("[STARTUP ERR] Error issuing teacher absence notification: {}", e),
        };
    }

    loop {
        println!("Iteration of notification loop");
        let current_period = get_current_period().await;

        match (current_period, prior_period.clone()) {
            (Some(ret), Some(cur)) if ret.id != cur.id => {
                prior_period = Some(ret);
                match issue_teacher_absence_notification(&cur).await {
                    Ok(_) => println!("Successfully issued teacher absence notification for period {}", &cur.id),
                    Err(e) => eprintln!("Error issuing teacher absence notification: {}", e),
                };
            },
            // A new day has started, current period is None and retrieved
            // period is Some, so we know a day has begun
            (Some(ret), None) => {
                println!("New day has likely started...");
                prior_period = Some(ret);
                match issue_teacher_absence_notification(&prior_period.clone().unwrap()).await {
                    Ok(_) => println!("Successfully issued teacher absence notification for period {}", &current_period.clone().unwrap().id),
                    Err(e) => eprintln!("Error issuing teacher absence notification: {}", e),
                };
            },
            
            _ => {
                // Do nothing, current period has not changed
                // Do nothing, no periods are currently happening
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
}

async fn get_current_period() -> Option<PeriodTeacherPeriods> {
    let data = match fetch_teacher_periods(period_teacher::Variables).await {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error fetching teacher periods: {:#?}", e);
            return None;
        }   
    };

    // Get the latest period end or 1 minute after midnight UTC as a fallback
    let last_period_end = data.periods.iter().map(|p| p.time_range.end).max_by(|a, b| a.total_cmp(b)).unwrap_or(60.0);

    let secs_since_midnight = chrono::Utc::now().time().num_seconds_from_midnight() as f64;

    // If current time is after end of the school day (last scheduled period)
    if secs_since_midnight > last_period_end {
        return None;
    }

    // Get the nearest period whose start time is less than 5 minutes from now
    // Helps avoid passing time/gaps between periods
    data.periods
        .into_iter()
        .filter(|p| p.time_range.start - PERIOD_ADVANCE <= secs_since_midnight)
        .max_by(|a, b| a.time_range.start.total_cmp(&b.time_range.start))
}