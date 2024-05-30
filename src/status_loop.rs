use std::collections::{HashSet, HashMap};
use std::error::Error;

use chrono::Timelike;

use crate::notifications::{
    NotificationDetails, NotificationType, 
    Topic, PeriodList,
    issue_notif,
};

use crate::gql::period_teacher::{
    PeriodTeacherPeriods,
    Variables as PeriodTeacherVariables,
};
use crate::gql::teachers::Variables as TeachersVariables;
use crate::gql::report_to::Variables as ReportToVariables;
use crate::gql::{fetch_report_to, fetch_teacher_periods, fetch_teachers};


/// The number of seconds ahead of time to look when determining the
/// upcoming/current period
const PERIOD_ADVANCE: f64 = 300.0;


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Absence {
    FullyAbsent,
    PartiallyAbsent { periods: HashSet<uuid::Uuid> },
    Present,
}


#[derive(Debug, Clone)]
pub struct TeacherState {
    pub name: String,
    pub absence: Absence,
    // pub periods: Vec<TeachersTeachersAbsence>,
    pub comments: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PeriodState {
    pub period_info: PeriodTeacherPeriods,
    pub teacher_info: HashMap<uuid::Uuid, TeacherState>,
}

pub async fn status_loop() -> Result<(), Box<dyn Error + Send>> {
    // let mut prior_state = get_current_state().await;
    let mut prior_state: Option<PeriodState> = None;

    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

        println!("Iteration of notification loop at [{:5}]", chrono::Utc::now().time().num_seconds_from_midnight() as f64);
        let current_period = get_current_state().await;
        let Some(curr_report_to) = get_curr_report_to().await else {
            continue;
        };

        match (current_period, prior_state.clone()) {
            // From during the day to during the day
            (Some(current), Some(prior)) => {
                if current.period_info.id == prior.period_info.id {
                    // Do nothing, current period has not changed
                    continue;
                }

                for notif in get_midday_notifs(&prior, &current, curr_report_to).await {
                    if let Err(e) = issue_notif(notif).await {
                        eprintln!("Error issuing notification: {}", e);
                    }
                }

                prior_state = Some(current);
            },

            // A new day has started, prior period is None and current
            // period is Some.
            (Some(ret), None) => {
                println!("New day has likely started...");

                for notif in get_begin_day_notifs(&ret, curr_report_to).await {
                    if let Err(e) = issue_notif(notif).await {
                        eprintln!("Error issuing notification: {}", e);
                    }
                }

                prior_state = Some(ret);
            },

            // Day has ended, set prior_period to None
            (None, Some(_)) => prior_state = None,
            
            // From not during the day to not during the day
            _ => {}
        }

    }
}

async fn get_curr_report_to() -> Option<String> {
    let data = match fetch_report_to(ReportToVariables).await {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error fetching current `report_to`: {:#?}", e);
            return None;
        }   
    };

    Some(data.report_to)
}

async fn get_all_periods() -> Option<Vec<PeriodTeacherPeriods>> {
    let data = match fetch_teacher_periods(PeriodTeacherVariables).await {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error fetching teacher periods: {:#?}", e);
            return None;
        }   
    };

    let mut periods = data.periods;

    periods.sort_by(|a, b| {
        a.time_range.start.total_cmp(&b.time_range.start)
    });

    Some(periods)
}

async fn get_current_period() -> Option<PeriodTeacherPeriods> {
    let data = match fetch_teacher_periods(PeriodTeacherVariables).await {
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

async fn get_current_teachers() -> Option<HashMap<uuid::Uuid, TeacherState>> {
    let data = match fetch_teachers(TeachersVariables).await {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error fetching teachers: {:#?}", e);
            return None;
        }   
    };

    let map = data.teachers
        .into_iter()
        .map(|t| (
            t.id,
            TeacherState {
                name: t.name.name,
                // periods: t.absence.clone(),
                absence: {
                    if t.fully_absent {
                        Absence::FullyAbsent
                    } else if t.absence.is_empty() {
                        Absence::Present
                    } else {
                        let mut periods: Vec<_> = t.absence;
                        periods.sort_by(|a, b| {
                            a.time_range.start.total_cmp(&b.time_range.start)
                        });
                        let periods = periods.into_iter().map(|p| p.id).collect();
                        Absence::PartiallyAbsent { periods }
                    }
                },
                comments: t.comments,
            },
        ))
        .collect();

    Some(map)
}

async fn get_current_state() -> Option<PeriodState> {
    let period_info = get_current_period().await;
    let teacher_info = get_current_teachers().await;

    match (period_info, teacher_info) {
        (Some(period_info), Some(teacher_info)) => {
            Some(PeriodState {
                period_info,
                teacher_info,
            })
        },
        _ => None,
    }
}




pub async fn get_midday_notifs(
    prior: &PeriodState,
    current: &PeriodState,
    report_to: String,
) -> impl Iterator<Item = NotificationDetails> {
    let Some(all_periods) = get_all_periods().await else {
        return vec![].into_iter();
    };

    let mut teacher_notifs = HashMap::new();

    // First, check for any teachers who's data has changed since last checked.
    // If a teacher's data has changed, we need to send a notification about
    // that.
    for (&teacher_id, teacher) in current.teacher_info.iter() {
        let prior_teacher = prior.teacher_info.get(&teacher_id);

        if let Some(prior_teacher) = prior_teacher {
            if prior_teacher.absence == teacher.absence { continue; }
        }
        let mut notifs = vec![];

        let id = uuid::Uuid::nil();
        let notification_type = match &teacher.absence {
            Absence::FullyAbsent => NotificationType::UpdateTeacherFullyAbsent,
            Absence::PartiallyAbsent { periods: period_ids } => {
                let period_inclusion_list: Vec<_> = all_periods
                    .iter()
                    .map(|p| (p.name.as_str(), period_ids.contains(&p.id)))
                    .collect();

                let periods = PeriodList::new(&period_inclusion_list);
                NotificationType::UpdateTeacherPartiallyAbsent { periods }
            },
            Absence::Present => NotificationType::UpdateTeacherPresent,
        };

        notifs.push(NotificationDetails {
            teacher_name: teacher.name.clone(),
            topic: Topic::from_teacher_and_period(
                &teacher_id.to_string(),
                &id.to_string(),
            ),
            notification_type,
            report_to: report_to.clone(),
            comments: teacher.comments.clone(),
        });
        teacher_notifs.insert(teacher_id, notifs);
    }

    // Next, check for any teachers who's period has changed since last checked.
    // If a teacher's period has changed, we need to send a notification about
    // the teacher being back in.
    let currently_absent: HashSet<_> = current.period_info.teachers_absent.iter().map(|t| t.id).collect();
    for teacher in prior.period_info.teachers_absent.iter() {
        if currently_absent.contains(&teacher.id) { continue; }

        let mut notifs = vec![];
        
        let id = uuid::Uuid::nil();
        notifs.push(NotificationDetails {
            teacher_name: teacher.name.normal.clone(),
            topic: Topic::from_teacher_and_period(
                &teacher.id.to_string(),
                &id.to_string(),
            ),
            notification_type: NotificationType::ReminderTeacherBackIn,
            report_to: report_to.clone(),
            comments: teacher.comments.clone(),
        });
        teacher_notifs.insert(teacher.id, notifs);
    }

    let notifs: Vec<_> = teacher_notifs.into_values().flatten().collect();

    notifs.into_iter()
}

pub async fn get_begin_day_notifs(current: &PeriodState, report_to: String) -> impl Iterator<Item = NotificationDetails> {
    let Some(all_periods) = get_all_periods().await else {
        return vec![].into_iter();
    };

    let mut teacher_notifs = HashMap::new();

    for (&teacher_id, teacher) in current.teacher_info.iter() {
        let mut notifs = vec![];

        let id = uuid::Uuid::nil();
        let notification_type = match &teacher.absence {
            Absence::FullyAbsent => NotificationType::DayStartFullyAbsent,
            Absence::PartiallyAbsent { periods: period_ids } => {

                let period_inclusion_list: Vec<_> = all_periods
                    .iter()
                    .map(|p| (p.name.as_str(), period_ids.contains(&p.id)))
                    .collect();

                let periods = PeriodList::new(&period_inclusion_list);
                NotificationType::DayStartPartiallyAbsent { periods }
            },
            Absence::Present => continue
        };

        notifs.push(NotificationDetails {
            teacher_name: teacher.name.clone(),
            topic: Topic::from_teacher_and_period(
                &teacher_id.to_string(),
                &id.to_string(),
            ),
            notification_type,
            report_to: report_to.clone(),
            comments: teacher.comments.clone(),
        });
        teacher_notifs.insert(teacher_id, notifs);
    }

    let notifs: Vec<_> = teacher_notifs.into_values().flatten().collect();

    notifs.into_iter()
}
