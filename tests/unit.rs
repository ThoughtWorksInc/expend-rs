extern crate expend;

mod per_diem {
    use expend::perdiem::TimePeriod;
    use expend::perdiem::TimePeriod::*;
    use expend::perdiem::Weekday::*;
    use std::str::FromStr;

    #[test]
    fn timeperiod_singleday_from_str_uppercase() {
        assert_eq!("Mon".parse().ok(), Some(SingleDay(Monday)));
        assert_eq!("Monday".parse().ok(), Some(SingleDay(Monday)));
    }

    #[test]
    fn timeperiod_singleday_from_str() {
        assert_eq!("mon".parse().ok(), Some(SingleDay(Monday)));
        assert_eq!("monday".parse().ok(), Some(SingleDay(Monday)));
        assert_eq!("tue".parse().ok(), Some(SingleDay(Tuesday)));
        assert_eq!("tuesday".parse().ok(), Some(SingleDay(Tuesday)));
        assert_eq!("wed".parse().ok(), Some(SingleDay(Wednesday)));
        assert_eq!("wednesday".parse().ok(), Some(SingleDay(Wednesday)));
        assert_eq!("thu".parse().ok(), Some(SingleDay(Thursday)));
        assert_eq!("thursday".parse().ok(), Some(SingleDay(Thursday)));
        assert_eq!("fri".parse().ok(), Some(SingleDay(Friday)));
        assert_eq!("friday".parse().ok(), Some(SingleDay(Friday)));
        assert_eq!("sat".parse().ok(), Some(SingleDay(Saturday)));
        assert_eq!("saturday".parse().ok(), Some(SingleDay(Saturday)));
        assert_eq!("sun".parse().ok(), Some(SingleDay(Sunday)));
        assert_eq!("sunday".parse().ok(), Some(SingleDay(Sunday)));
    }

    #[test]
    fn timeperiod_dayrange_from_str() {
        assert_eq!(
            "mon-saturday".parse().ok(),
            Some(DayRange {
                from: Monday,
                to: Saturday
            })
        );
        assert_eq!(
            "tuesday-wednesday".parse().ok(),
            Some(DayRange {
                from: Tuesday,
                to: Wednesday
            })
        );
        assert!(TimePeriod::from_str("wednesday-tue").is_err());
        assert_eq!("thu-thursday".parse().ok(), Some(SingleDay(Thursday)));
        assert!(TimePeriod::from_str("mon-saturday-tue").is_err());
    }
}
