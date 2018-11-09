use Weekday;
use failure::{bail, Error};
use std::str::FromStr;

#[derive(Debug, PartialEq, Eq)]
pub enum TimePeriod {
    Weekdays,
    SingleDay(Weekday),
    DayRange { from: Weekday, to: Weekday },
    Days(Vec<Weekday>),
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
                    let days = commas.into_iter().fold(Vec::new(), |mut acc, d| {
                        if !acc.iter().any(|od| *od == d) {
                            acc.push(d)
                        }
                        acc
                    });
                    match days.as_slice() {
                        [] => bail!("Didn't see a single weekday in '{}'", s),
                        &[d] => SingleDay(d),
                        &[from, to] if from.numerical() == to.numerical() - 1 => {
                            DayRange { from, to }
                        }
                        _ => Days(days.clone()),
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
