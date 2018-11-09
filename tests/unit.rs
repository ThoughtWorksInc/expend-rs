extern crate expend;

mod per_diem {
    mod timeperiod {
        use expend::{TimePeriod, TimePeriod::*};
        use expend::Weekday::*;
        use std::str::FromStr;

        #[test]
        fn singleday_from_str_uppercase() {
            assert_eq!("Mon".parse().ok(), Some(SingleDay(Monday)));
            assert_eq!("Monday".parse().ok(), Some(SingleDay(Monday)));
        }

        #[test]
        fn singleday_from_str_whitespace() {
            assert_eq!("  Mon".parse().ok(), Some(SingleDay(Monday)));
            assert_eq!("Monday  ".parse().ok(), Some(SingleDay(Monday)));
        }

        #[test]
        fn singleday_from_str() {
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
        fn dayrange_from_str_whitespace() {
            assert_eq!(
                "  mon  -  saturday  ".parse().ok(),
                Some(DayRange {
                    from: Monday,
                    to: Saturday
                })
            );
        }

        #[test]
        fn dayrange_from_str_from_consequitive_days() {
            assert_eq!(
                "mon,tue".parse().ok(),
                Some(DayRange {
                    from: Monday,
                    to: Tuesday
                })
            );
        }

        #[test]
        fn dayrange_from_str() {
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
        }

        #[test]
        fn dayrange_from_str_not_enough_days() {
            assert_eq!(TimePeriod::from_str("mon- ").ok(), Some(SingleDay(Monday)));
        }

        #[test]
        fn dayrange_from_str_too_many_days() {
            assert!(TimePeriod::from_str("mon-saturday-tue").is_err());
        }

        #[test]
        fn dayrange_from_str_same_day() {
            assert_eq!("thu-thursday".parse().ok(), Some(SingleDay(Thursday)));
        }

        #[test]
        fn dayrange_from_str_skip_empties() {
            assert_eq!("thu- - - Thursday".parse().ok(), Some(SingleDay(Thursday)));
        }

        #[test]
        fn dayrange_from_str_invalid_order() {
            assert!(TimePeriod::from_str("wednesday-tue").is_err());
        }

        #[test]
        fn anydays_from_str() {
            assert_eq!(
                "mon,tuesday,sun".parse().ok(),
                Some(Days(vec![Monday, Tuesday, Sunday]))
            );
        }

        #[test]
        fn anydays_from_str_whitespace() {
            assert_eq!(
                " tuesday, Saturday ".parse().ok(),
                Some(Days(vec![Tuesday, Saturday]))
            );
        }

        #[test]
        fn anydays_from_str_reorder() {
            assert_eq!(
                " Saturday, Monday, Wednesday, Sunday ".parse().ok(),
                Some(Days(vec![Monday, Wednesday, Saturday, Sunday]))
            );
        }

        #[test]
        fn anydays_from_str_duplicates() {
            assert_eq!("Sunday, sun".parse().ok(), Some(SingleDay(Sunday)));
        }

        #[test]
        fn anydays_from_str_empty_commas() {
            assert!(TimePeriod::from_str(", , ,").is_err());
        }

        #[test]
        fn anydays_from_str_skip_empty() {
            assert_eq!("mon, , ,sun".parse().ok(), Some(Days(vec![Monday, Sunday])));
        }
    }
}
