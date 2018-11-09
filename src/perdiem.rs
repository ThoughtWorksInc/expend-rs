use chrono::{Date, Utc};
use context::{Country, Country::*};
use expensify::{TransactionList, TransactionListElement};
use failure::Error;
use std::str::FromStr;
use {Context, EXPENSIFY_DATE_FORMAT};
use TimePeriod;
use std::fmt;

impl TransactionList {
    pub fn from_per_diem(ctx: Context, period: TimePeriod, kind: Kind) -> Result<Self, Error> {
        Ok(TransactionList {
            transaction_list_type: "expenses".to_owned(),
            employee_email: ctx.user.email.clone(),
            transaction_list: period.into_transactions(&ctx, kind)?,
        })
    }
}

pub enum Kind {
    FullDay,
    Breakfast,
    Arrival,
    Departure,
    Daytrip,
    Lunch,
    Dinner
}

impl Kind {
    fn amount(&self, c: &Country) -> u32 {
        use self::Kind::*;
        (match c {
            Germany => match self {
                FullDay => 240,
                Breakfast => 48,
                Daytrip|Arrival|Departure => 120,
                Lunch|Dinner => 96
            }
        }) * 10
    }
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        use self::Kind::*;
        match self {
            FullDay => f.write_str("Full Day"),
            Breakfast => f.write_str("Breakfast"),
            Arrival|Departure => f.write_str("Arrival/Departure Day"),
            Daytrip => f.write_str("Day Trip > 8 Hours"),
            Lunch|Dinner => f.write_str("Lunch/Dinner"),
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

fn to_merchant(num_days: u32, ctx: &Context, kind: &Kind) -> String {
    format!(
        "{} * {} {} @ {}{:.2}",
        num_days,
        ctx.user.country,
        kind,
        ctx.user.country.currency().symbol(),
        (kind.amount(&ctx.user.country) / 10) as f32 / 10.0
    )
}

fn to_element_from_range(
    from: &Date<Utc>,
    to: &Date<Utc>,
    ctx: &Context,
    kind: &Kind,
) -> TransactionListElement {
    let num_days = (*to - *from).num_days() + 1;
    assert!(num_days > 0, "to-date must be larger than from-date");

    to_element(
        to_date_string(&from),
        to_comment_from_range(&from, &to),
        num_days as u32,
        ctx,
        &kind,
    )
}
fn to_element(
    created: String,
    comment: String,
    num_days: u32,
    ctx: &Context,
    kind: &Kind,
) -> TransactionListElement {
    TransactionListElement {
        created,
        currency: format!("{}", ctx.user.country.currency()),
        merchant: to_merchant(num_days, ctx, &kind),
        amount: (kind.amount(&ctx.user.country) * num_days) as i64,
        category: String::new(),
        tag: format!("{}:{}", ctx.user.project.clone(), ctx.user.tags.travel.name),
        billable: ctx.user.tags.travel.billable,
        reimbursable: true,
        comment,
    }
}

fn to_comment_from_range(from: &Date<Utc>, to: &Date<Utc>) -> String {
    format!("{} to {}", to_date_string(&from), to_date_string(&to))
}

fn to_element_single_day(day: &Date<Utc>, ctx: &Context, kind: &Kind) -> TransactionListElement {
    to_element(to_date_string(day), "".to_string(), 1, ctx, &kind)
}

impl TimePeriod {
    fn into_transactions(
        self,
        ctx: &Context,
        kind: Kind,
    ) -> Result<Vec<TransactionListElement>, Error> {
        use TimePeriod::*;
        use Weekday::Friday;

        let mut ts = Vec::new();
        let monday = ctx.monday_of_reference_date()?;
        match self {
            Weekdays => {
                let friday = Friday.to_date_from(&monday)?;
                ts.push(to_element_from_range(&monday, &friday, ctx, &kind));
            }
            SingleDay(day) => {
                let day = day.to_date_from(&monday)?;
                ts.push(to_element_single_day(&day, ctx, &kind));
            }
            DayRange { from, to } => {
                let from = from.to_date_from(&monday)?;
                let to = to.to_date_from(&monday)?;
                ts.push(to_element_from_range(&from, &to, ctx, &kind));
            }
            Days(d) => for day in d {
                let day = day.to_date_from(&monday)?;
                ts.push(to_element_single_day(&day, ctx, &kind));
            },
        }
        Ok(ts)
    }
}
