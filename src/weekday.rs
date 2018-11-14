use chrono::prelude::*;
use failure::Error;
use std::{fmt, str::FromStr};
use time::Duration;

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
    pub fn to_date_from(&self, reference: &Date<Utc>) -> Result<Date<Utc>, Error> {
        reference
            .checked_add_signed(Duration::days(self.numerical() as i64))
            .ok_or_else(|| {
                format_err!(
                    "Failed to compute day {} from reference date {}",
                    self,
                    reference
                )
            })
    }

    pub fn is_after(&self, other: &Weekday) -> bool {
        self.numerical() > other.numerical()
    }

    pub fn numerical(&self) -> u8 {
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
