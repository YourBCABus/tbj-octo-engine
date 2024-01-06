use super::NotificationType;

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
}

impl From<NotificationDetails> for NotificationPayload {
    fn from(value: NotificationDetails) -> Self {
        use NotificationType::*;

        match value.notification_type {
            DayStartFullyAbsent => Self {
                topic: value.topic,
                title: format!("{} Absent", value.teacher_name),
                body: Some(format!("{} is absent today. Please sign into upper caf.", value.teacher_name)),
            },
            DayStartPartiallyAbsent { periods } => Self {
                topic: value.topic,
                title: format!("{} Partially Absent", value.teacher_name),
                body: Some(format!("{} is absent today during period(s) {}. Please sign into upper caf.", value.teacher_name, periods)),
            },
            UpdateTeacherFullyAbsent => Self {
                topic: value.topic,
                title: format!("{} Absent", value.teacher_name),
                body: Some(format!("{} will now be absent for the rest of the day. Please sign into upper caf.", value.teacher_name)),
            },
            UpdateTeacherPartiallyAbsent { periods } => Self {
                topic: value.topic,
                title: format!("{} Partially Absent", value.teacher_name),
                body: Some(format!("{} will not be here for period(s) {}. Please sign into upper caf.", value.teacher_name, periods)),
            },
            UpdateTeacherPresent => Self {
                topic: value.topic,
                title: format!("{} is Present", value.teacher_name),
                body: Some(format!("{} is back in school. Please report to class.", value.teacher_name)),
            },
            ReminderTeacherBackIn => Self {
                topic: value.topic,
                title: format!("{} is Present", value.teacher_name),
                body: Some(format!("Reminder: {} is back. Please report to class.", value.teacher_name)),
            },
        }
    }
}
