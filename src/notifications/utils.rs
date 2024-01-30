use std::fmt::Display;



#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeriodList(EnglishList<PeriodGroup>);

impl PeriodList {
    pub fn new(periods: &[(&str, bool)]) -> Self {
        let mut groups = vec![];
        let mut curr_range = None;
        for (idx, (_, is_included)) in periods.iter().copied().chain(std::iter::once(("", false))).enumerate() {
            if is_included {
                if let Some((start, _)) = curr_range {
                    curr_range = Some((start, idx + 1));
                } else {
                    curr_range = Some((idx, idx + 1));
                }
            } else if let Some((start, end)) = curr_range {
                if end - start == 1 {
                    groups.push(PeriodGroup::SinglePeriod(trim_period_text(periods[start].0).to_string()));
                } else {
                    groups.push(PeriodGroup::PeriodRange {
                        start: trim_period_text(periods[start].0).to_string(),
                        end: trim_period_text(periods[end - 1].0).to_string(),
                    });
                }
                curr_range = None;
            }
        }

        Self(EnglishList(groups))
    }
}

impl Display for PeriodList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

fn trim_period_text(period_name: &str) -> &str {
    period_name
        .trim()
        .trim_start_matches("Period").trim_start_matches("period")
        .trim_end_matches("Period").trim_end_matches("period")
        .trim()
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PeriodGroup {
    SinglePeriod(String),
    PeriodRange {
        start: String,
        end: String,
    },
}

impl Display for PeriodGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PeriodGroup::SinglePeriod(period) => write!(f, "{}", period),
            PeriodGroup::PeriodRange { start, end } => write!(f, "{}-{}", start, end),
        }
    }
}



#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnglishList<T: Display>(Vec<T>);

impl<T: Display> Display for EnglishList<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0.len() {
            0 => Ok(()),
            1 => write!(f, "{}", self.0[0]),
            2 => write!(f, "{} and {}", self.0[0], self.0[1]),
            _ => {
                for (i, item) in self.0.iter().enumerate() {
                    if i == self.0.len() - 1 {
                        write!(f, "and {}", item)?;
                    } else {
                        write!(f, "{}, ", item)?;
                    }
                }
                Ok(())
            }
        }
    }
}
