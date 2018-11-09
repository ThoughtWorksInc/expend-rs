use chrono::{Date, Utc};
use context::Country;
use context::Country::*;
use expensify::{TransactionList, TransactionListElement};
use failure::Error;
use std::{fmt, str::FromStr};
use time::Duration;
use {Context, EXPENSIFY_DATE_FORMAT};

impl TransactionList {
    pub fn from_per_diem(ctx: Context, period: TimePeriod, kind: Kind) -> Result<Self, Error> {
        Ok(TransactionList {
            transaction_list_type: "expenses".to_owned(),
            employee_email: ctx.user.email.clone(),
            transaction_list: period.into_transactions(&ctx, kind)?,
        })
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

#[derive(Debug, PartialEq, Eq)]
pub enum TimePeriod {
    Weekdays,
    SingleDay(Weekday),
    DayRange { from: Weekday, to: Weekday },
    Days(Vec<Weekday>),
}

impl FromStr for Weekday {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        use self::Weekday::*;
        Ok(match s.to_ascii_lowercase().as_str() {
            "mon" | "monday" => Monday,
            "tue" | "tuesday" => Tuesday,
            "wed" | "wednesday" => Wednesday,
            "thu" | "thursday" => Thursday,
            "fri" | "friday" => Friday,
            "sat" | "saturday" => Saturday,
            "sun" | "sunday" => Sunday,
            _ => bail!("Invalid weekday specification: '{}'", s),
        })
    }
}

impl Weekday {
    fn is_after(&self, other: &Weekday) -> bool {
        self.numerical() > other.numerical()
    }

    fn numerical(&self) -> u8 {
        use self::Weekday::*;
        match self {
            Monday => 0,
            Tuesday => 1,
            Wednesday => 2,
            Thursday => 3,
            Friday => 4,
            Saturday => 5,
            Sunday => 6,
        }
    }
}

impl fmt::Display for Weekday {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        use self::Weekday::*;
        match self {
            Monday => f.write_str("Monday"),
            Tuesday => f.write_str("Tuesday"),
            Wednesday => f.write_str("Wednesday"),
            Thursday => f.write_str("Thursday"),
            Friday => f.write_str("Friday"),
            Saturday => f.write_str("Saturday"),
            Sunday => f.write_str("Sunday"),
        }
    }
}

impl FromStr for TimePeriod {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        use self::TimePeriod::*;
        let mut words = s.trim().split('-').map(str::trim).filter(|s| s.len() != 0);

        Ok(match (words.next(), words.next(), words.next()) {
            (Some(first), None, None) => match first {
                "weekdays" => Weekdays,
                s => {
                    let mut commas: Vec<_> = s.split(',')
                        .map(str::trim)
                        .filter(|s| s.len() != 0)
                        .map(Weekday::from_str)
                        .collect::<Result<_, _>>()?;
                    commas.sort_by_key(|d| d.numerical());
                    let commas = commas.into_iter().fold(Vec::new(), |mut acc, d| {
                        if !acc.iter().any(|od| *od == d) {
                            acc.push(d)
                        }
                        acc
                    });
                    match commas.len() {
                        0 => bail!("Didn't see a single weekday in '{}'", s),
                        1 => SingleDay(commas[0]),
                        _ => Days(commas),
                    }
                }
            },
            (Some(first), Some(second), None) => {
                let from = first.parse()?;
                let to = second.parse()?;
                if from == to {
                    SingleDay(from)
                } else if from.is_after(&to) {
                    bail!("Day '{}' must be temporally before '{}', but came after. Write '{}-{}' instead.", from, to, to, from)
                } else {
                    DayRange { from, to }
                }
            }
            _ => bail!(
                "More than two days separated by '-' are not allowed in '{}'",
                s
            ),
        })
    }
}

pub enum Kind {
    FullDay,
}

impl Kind {
    fn amount(&self, c: &Country) -> u32 {
        (match c {
            Germany => 24,
        }) * 100
    }
}

impl FromStr for Kind {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        use self::Kind::*;
        Ok(match s {
            "fullday" => FullDay,
            _ => bail!("Invalid per diem kind specification: '{}'", s),
        })
    }
}

fn to_date_string(d: &Date<Utc>) -> String {
    d.format(EXPENSIFY_DATE_FORMAT).to_string()
}

impl TimePeriod {
    fn into_transactions(
        self,
        ctx: &Context,
        kind: Kind,
    ) -> Result<Vec<TransactionListElement>, Error> {
        use self::TimePeriod::*;

        let mut ts = Vec::new();
        match self {
            Weekdays => {
                let monday = ctx.monday_of_reference_date()?;
                let friday = monday.checked_add_signed(Duration::days(5 - 1)).unwrap();
                let num_days = 5;

                ts.push(TransactionListElement {
                    created: to_date_string(&monday),
                    currency: format!("{}", ctx.user.country.currency()),
                    merchant: format!(
                        "{} * {} Full Day @ {}{:.2}",
                        num_days,
                        ctx.user.country,
                        ctx.user.country.currency().symbol(),
                        (kind.amount(&ctx.user.country) / 100) as f32
                    ),
                    amount: (kind.amount(&ctx.user.country) * num_days) as i64,
                    category: String::new(),
                    tag: format!("{}:{}", ctx.user.project.clone(), ctx.user.tags.travel.name),
                    billable: ctx.user.tags.travel.billable,
                    reimbursable: true,
                    comment: format!("{} to {}", to_date_string(&monday), to_date_string(&friday)),
                });
            }
            SingleDay(_day) => unimplemented!("Single-Day"),
            DayRange { from: _, to: _ } => unimplemented!("day range"),
            Days(_d) => unimplemented!("days"),
        }
        Ok(ts)
    }
}
