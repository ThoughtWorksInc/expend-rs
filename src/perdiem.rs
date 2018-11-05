use chrono::{Date, Utc};
use failure::Error;
use std::str::FromStr;
use time::Duration;
use types::{TransactionList, TransactionListElement};
use {Context, EXPENSIFY_DATE_FORMAT};

impl TransactionList {
    pub fn from_per_diem(ctx: Context, kind: PerDiem) -> Result<Self, Error> {
        Ok(TransactionList {
            transaction_list_type: "expenses".to_owned(),
            employee_email: ctx.user.email.clone(),
            transaction_list: kind.into_transactions(&ctx)?,
        })
    }
}

pub enum PerDiem {
    Weekdays,
}

impl FromStr for PerDiem {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        use self::PerDiem::*;
        Ok(match s {
            "weekdays" => Weekdays,
            _ => bail!("Invalid per diem specification: '{}'", s),
        })
    }
}

fn to_date_string(d: &Date<Utc>) -> String {
    d.format(EXPENSIFY_DATE_FORMAT).to_string()
}

impl PerDiem {
    fn into_transactions(self, ctx: &Context) -> Result<Vec<TransactionListElement>, Error> {
        use self::PerDiem::*;

        let mut ts = Vec::new();
        match self {
            Weekdays => {
                let monday = ctx.monday_of_reference_date()?;
                let friday = monday.checked_add_signed(Duration::days(5 - 1)).unwrap();

                ts.push(TransactionListElement {
                    created: to_date_string(&monday),
                    currency: String::new(),
                    merchant: String::new(),
                    amount: 0,
                    category: String::new(),
                    tag: ctx.user.project.clone(),
                    billable: false,
                    reimbursable: false,
                    comment: format!("{} to {}", to_date_string(&monday), to_date_string(&friday)),
                });
            }
        }
        Ok(ts)
    }
}
