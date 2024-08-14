use super::NotificationType;
use std::fmt::Write;

#[derive(Debug, Clone)]
pub struct Topic(String);
impl Topic {
    pub fn from_teacher_and_period(teacher: &str, period: &str) -> Self {
        Self(format!("{period}.{teacher}"))
    }
}

impl std::fmt::Display for Topic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "/topics/{}", self.0)
    }
}


#[derive(Debug, Clone)]
pub struct NotificationDetails {
    pub teacher_name: String,
    pub topic: Topic,
    pub notification_type: NotificationType,
    pub report_to: String,
    pub comments: Option<String>,
}

impl NotificationDetails {
    pub fn build(self) -> NotificationPayload {
        self.into()
    }
}

#[derive(Debug)]
pub struct NotificationPayload {
    pub topic: Topic,
    pub title: String,
    pub body: Option<String>,
    pub analytics: Option<String>,
}

impl From<NotificationDetails> for NotificationPayload {
    fn from(value: NotificationDetails) -> Self {
        use NotificationType::*;

        let mut body = match &value.notification_type {
            DayStartFullyAbsent => format!("{} is absent today", value.teacher_name),
            DayStartPartiallyAbsent { periods } => format!("{} is absent today during period(s) {}", value.teacher_name, periods),
            UpdateTeacherFullyAbsent => format!("{} will now be absent for the rest of the day", value.teacher_name),
            UpdateTeacherPartiallyAbsent { periods } => format!("{} will not be here for period(s) {}", value.teacher_name, periods),
            UpdateTeacherPresent => format!("{} is back in school", value.teacher_name),
            ReminderTeacherBackIn => format!("Reminder: {} is back", value.teacher_name),
        };

        if matches!(
            value.notification_type,
            DayStartFullyAbsent | DayStartPartiallyAbsent { .. } | UpdateTeacherFullyAbsent | UpdateTeacherPartiallyAbsent { .. },
        ) {
            if value.comments.is_some() {
                write!(&mut body, ". YOUR TEACHER HAS LEFT ADDITIONAL COMMENTS IN THE APP. Please also check Schoology.").unwrap();
            } else {
                write!(&mut body, ". Check Schoology and report to {}.", value.report_to).unwrap();
            }
        }

        let title = match value.notification_type {
            DayStartFullyAbsent | UpdateTeacherFullyAbsent => format!("{} Absent", value.teacher_name),
            DayStartPartiallyAbsent { .. } | UpdateTeacherPartiallyAbsent { .. } => format!("{} PARTIALLY Absent", value.teacher_name),
            UpdateTeacherPresent | ReminderTeacherBackIn => format!("{} is Present", value.teacher_name),
        };

        Self {
            topic: value.topic,
            title,
            body: Some(body),
            analytics: Some(value.notification_type.analytics_label().to_string()),
        }
    }
}
