use chrono::{Date, Utc};
use context::{Country, Country::*};
use expensify::{TransactionList, TransactionListElement};
use failure::Error;
use std::str::FromStr;
use {Context, EXPENSIFY_DATE_FORMAT};
use TimePeriod;

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

fn to_merchant(num_days: u32, ctx: &Context, kind: &Kind) -> String {
    format!(
        "{} * {} Full Day @ {}{:.2}",
        num_days,
        ctx.user.country,
        ctx.user.country.currency().symbol(),
        (kind.amount(&ctx.user.country) / 100) as f32
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
                let num_days = 5;

                ts.push(to_element(
                    to_date_string(&monday),
                    to_comment_from_range(&monday, &friday),
                    num_days,
                    ctx,
                    &kind,
                ));
            }
            SingleDay(day) => {
                let day = day.to_date_from(&monday)?;
                ts.push(to_element_single_day(&day, ctx, &kind));
            }
            DayRange { from, to } => {
                let num_days = to.numerical()
                    .checked_sub(from.numerical())
                    .expect("to-date to be larger than from-date")
                    + 1;
                let from = from.to_date_from(&monday)?;
                let to = to.to_date_from(&monday)?;
                ts.push(to_element(
                    to_date_string(&from),
                    to_comment_from_range(&from, &to),
                    num_days.into(),
                    ctx,
                    &kind,
                ));
            }
            Days(d) => for day in d {
                let day = day.to_date_from(&monday)?;
                ts.push(to_element_single_day(&day, ctx, &kind));
            },
        }
        Ok(ts)
    }
}
