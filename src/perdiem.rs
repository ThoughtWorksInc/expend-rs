use chrono::{Date, Utc};
use context::{Country, Country::*, Destination};
use expensify::{TransactionList, TransactionListElement};
use failure::Error;
use std::fmt;
use std::str::FromStr;
use TimePeriod;
use {Context, EXPENSIFY_DATE_FORMAT};

impl TransactionList {
    pub fn from_per_diem(
        ctx: Context,
        period: TimePeriod,
        kind: Kind,
        mode: Mode,
    ) -> Result<Self, Error> {
        Ok(TransactionList {
            transaction_list_type: "expenses".to_owned(),
            employee_email: ctx.user.email.clone(),
            transaction_list: period.into_transactions(&ctx, kind, mode)?,
        })
    }
}

pub enum Mode {
    Add,
    Subtract,
}

impl<'a> ::std::ops::Mul<&'a Mode> for i32 {
    type Output = i32;

    fn mul(self, rhs: &'a Mode) -> i32 {
        match rhs {
            Mode::Add => self,
            Mode::Subtract => self * -1,
        }
    }
}

pub enum Kind {
    FullDay,
    Breakfast,
    Arrival,
    Departure,
    Daytrip,
    Lunch,
    Dinner,
}

impl Kind {
    fn amount(&self, c: &Country, d: Option<&Destination>) -> u32 {
        use self::Kind::*;
        let res = (|| {
            Ok((match (c, d) {
                (Germany, Some(Destination::IndiaOther)) => match self {
                    FullDay => 320,
                    Daytrip | Arrival | Departure => 210,
                    anything_else => return Err(anything_else),
                },
                (Germany, None) => match self {
                    FullDay => 240,
                    Daytrip | Arrival | Departure => 120,
                    Breakfast => 48,
                    Lunch | Dinner => 96,
                },
            }) * 10)
        })();
        match res {
            Ok(val) => val,
            // It's not good for a lib to panic, but I find it too cumbersome to pass-through the error through all callers right now.
            // Maybe eventually we will learn how to produce a valid value in all cases.
            Err(invalid_kind) => panic!(
                "The per-diem of kind '{}' cannot yield an amount - please try another one.",
                invalid_kind
            ),
        }
    }
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        use self::Kind::*;
        match self {
            FullDay => f.write_str("Full Day"),
            Breakfast => f.write_str("Breakfast"),
            Arrival | Departure => f.write_str("Arrival/Departure Day"),
            Daytrip => f.write_str("Day Trip > 8 Hours"),
            Lunch | Dinner => f.write_str("Lunch/Dinner"),
        }
    }
}

impl FromStr for Kind {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        use self::Kind::*;
        Ok(match s.to_ascii_lowercase().as_str() {
            "fullday" => FullDay,
            "breakfast" => Breakfast,
            "arrival" => Arrival,
            "departure" => Departure,
            "daytrip" => Daytrip,
            "lunch" => Lunch,
            "dinner" => Dinner,
            _ => bail!("Invalid per diem kind specification: '{}'", s),
        })
    }
}

fn to_date_string(d: &Date<Utc>) -> String {
    d.format(EXPENSIFY_DATE_FORMAT).to_string()
}

fn to_merchant(num_days: u32, ctx: &Context, kind: &Kind, mode: &Mode) -> String {
    format!(
        "{} * {}{} {} @ {}{:.2}",
        num_days,
        match ctx.user.destination {
            Some(ref destination) => format!("{} for ", destination),
            None => "".to_owned(),
        },
        ctx.user.country,
        kind,
        ctx.user.country.currency().symbol(),
        ((kind.amount(&ctx.user.country, ctx.user.destination.as_ref()) / 10) as i32 * mode) as f32
            / 10.0
    )
}

fn to_element_from_range(
    from: &Date<Utc>,
    to: &Date<Utc>,
    ctx: &Context,
    kind: &Kind,
    mode: &Mode,
) -> TransactionListElement {
    let num_days = (*to - *from).num_days() + 1;
    assert!(num_days > 0, "to-date must be larger than from-date");
    let comment = to_comment_from_range(&from, &to);

    to_element(
        to_date_string(&from),
        ctx.comment
            .as_ref()
            .map(|custom| format!("{}: {}", comment, custom))
            .unwrap_or(comment),
        num_days as u32,
        ctx,
        kind,
        mode,
    )
}
fn to_element(
    created: String,
    comment: String,
    num_days: u32,
    ctx: &Context,
    kind: &Kind,
    mode: &Mode,
) -> TransactionListElement {
    TransactionListElement {
        created,
        currency: format!("{}", ctx.user.country.currency()),
        merchant: to_merchant(num_days, ctx, &kind, mode),
        amount: (kind.amount(&ctx.user.country, ctx.user.destination.as_ref()) * num_days) as i32
            * mode,
        category: ctx.user.categories.per_diems.name.clone(),
        tag: format!("{}:{}", ctx.user.project.clone(), ctx.user.tags.travel.name),
        billable: ctx.user.tags.travel.billable,
        reimbursable: true,
        comment,
    }
}

fn to_comment_from_range(from: &Date<Utc>, to: &Date<Utc>) -> String {
    format!("{} to {}", to_date_string(&from), to_date_string(&to))
}

fn to_element_single_day(
    day: &Date<Utc>,
    ctx: &Context,
    kind: &Kind,
    mode: &Mode,
) -> TransactionListElement {
    to_element(
        to_date_string(day),
        ctx.comment.clone().unwrap_or_default(),
        1,
        ctx,
        kind,
        mode,
    )
}

impl TimePeriod {
    fn into_transactions(
        self,
        ctx: &Context,
        kind: Kind,
        mode: Mode,
    ) -> Result<Vec<TransactionListElement>, Error> {
        use TimePeriod::*;
        use Weekday::Friday;

        let mut ts = Vec::new();
        let monday = ctx.monday_of_reference_date()?;
        match self {
            Weekdays => {
                let friday = Friday.to_date_from(&monday)?;
                ts.push(to_element_from_range(&monday, &friday, ctx, &kind, &mode));
            }
            SingleDay(day) => {
                let day = day.to_date_from(&monday)?;
                ts.push(to_element_single_day(&day, ctx, &kind, &mode));
            }
            DayRange { from, to } => {
                let from = from.to_date_from(&monday)?;
                let to = to.to_date_from(&monday)?;
                ts.push(to_element_from_range(&from, &to, ctx, &kind, &mode));
            }
            Days(d) => {
                for day in d {
                    let day = day.to_date_from(&monday)?;
                    ts.push(to_element_single_day(&day, ctx, &kind, &mode));
                }
            }
        }
        Ok(ts)
    }
}
