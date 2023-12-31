use super::NotificationType;

#[derive(Debug, Clone)]
pub struct Topic(pub String);
impl Topic {
    pub fn from_teacher(id: &str, default_period_id: &str) -> Self {
        Self(format!("{default_period_id}.{id}"))
    }
    pub fn from_teacher_and_period(teacher: &str, period: &str) -> Self {
        Self(format!("{period}.{teacher}"))
    }
}

impl std::fmt::Display for Topic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}


#[derive(Debug, Clone)]
pub struct NotificationDetails<'a> {
    pub teacher_name: String,
    pub topic: Topic,
    pub notification_type: NotificationType<'a>,
}

impl<'a> NotificationDetails<'a> {
    pub fn build(self) -> NotificationPayload {
        self.into()
    }
}

#[derive(Debug)]
pub struct NotificationPayload {
    topic: Topic,
    title: String,
    body: Option<String>,
}

impl From<NotificationDetails<'_>> for NotificationPayload {
    fn from(value: NotificationDetails<'_>) -> Self {
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
                body: Some(format!("{} is absent today during period(s) {}. Please sign into upper caf.", value.teacher_name, periods.join(", "))),
            },
            UpdateTeacherFullyAbsent => Self {
                topic: value.topic,
                title: format!("{} Absent", value.teacher_name),
                body: Some(format!("{} will now be absent for the rest of the day. Please sign into upper caf.", value.teacher_name)),
            },
            UpdateTeacherPartiallyAbsent { periods } => Self {
                topic: value.topic,
                title: format!("{} Partially Absent", value.teacher_name),
                body: Some(format!("{} will not be here for  period(s) {}. Please sign into upper caf.", value.teacher_name, periods.join(", "))),
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
