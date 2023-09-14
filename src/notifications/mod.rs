use std::{error::Error, thread::current};

use chrono::Timelike;
use fcm::{Client, MessageBuilder, NotificationBuilder};

use crate::gql::{fetch_teacher_periods, period_teacher::{self, ResponseData, PeriodTeacherPeriods}};

#[derive(Debug)]
pub struct NotificationPayload {
    topic: String,
    teacher_name: String,
    body: Option<String>,
}

pub async fn notify(payload: &NotificationPayload) {
    let api_key = dotenvy::var("FIREBASE_API_KEY").unwrap();

    let client = Client::new();

    let mut builder = NotificationBuilder::new();
    let body = format!("{} is absent this period. Sign into the upper cafeteria.", payload.teacher_name);
    let title = format!("{} is absent", payload.teacher_name);

    builder.title(&title);
    builder.body(&body);
    let notification = builder.finalize();

    let mut builder = MessageBuilder::new(api_key.as_str(), &payload.topic);
    builder.notification(notification);
    let message = builder.finalize();

    let response = client.send(message).await.unwrap();
    println!("Sent: {:?}", response);
}    

pub async fn issue_teacher_absence_notification(current_period: &PeriodTeacherPeriods) -> Result<(), Box<dyn Error>> {
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
    ).await;
    Ok(())
}

pub async fn notification_loop() -> Result<(), Box<dyn Error>> {
    let mut current_period = get_current_period().await;

    if let Some(cur) = current_period.clone() {
        match issue_teacher_absence_notification(&cur).await {
            Ok(_) => println!("[STARTUP NOTIF] Successfully issued teacher absence notification for period {:#?}", &cur),
            Err(e) => eprintln!("[STARTUP ERR] Error issuing teacher absence notification: {}", e),
        };
    }

    loop {
        println!("Iteration of notification loop");
        let retrieved_period = get_current_period().await;

        match (retrieved_period, current_period.clone()) {
            (Some(ret), Some(cur)) => {
                if ret.id != cur.id {
                    current_period = Some(ret);
                    match issue_teacher_absence_notification(&cur).await {
                        Ok(_) => println!("Successfully issued teacher absence notification for period {}", &cur.id),
                        Err(e) => eprintln!("Error issuing teacher absence notification: {}", e),
                    };
                }
            },
            (Some(ret), None) => {
                println!("New day has likely started...");
                current_period = Some(ret);
                match issue_teacher_absence_notification(&current_period.clone().unwrap()).await {
                    Ok(_) => println!("Successfully issued teacher absence notification for period {}", &current_period.clone().unwrap().id),
                    Err(e) => eprintln!("Error issuing teacher absence notification: {}", e),
                };
            },
            (None, Some(_cur)) => {
                // Do nothing, current period has not changed
            },
            (None, None) => {
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

    let time_since_midnight = chrono::Utc::now().time().num_seconds_from_midnight() as f64;

    println!("Time secs: {}", time_since_midnight);
    let current_period = data.periods
        .into_iter()
        .find(
            |period| 
                (period.time_range.start < time_since_midnight) &&
                (time_since_midnight < period.time_range.end)
        );
    
    if let Some(period) = current_period {
        return Some(period);
    } else {
        eprintln!("Error getting current period");
        return None;
    }
}