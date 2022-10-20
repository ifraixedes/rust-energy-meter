use crate::utils;

use std::collections::hash_map::HashMap;
use std::collections::hash_set::HashSet;
use std::vec::Vec;

use chrono::naive::NaiveDate;

pub struct Cmd {
    bank_holidays: HashSet<String>,
    period_bank_holidays: u8,
    counters: HashMap<u8, u64>,
    periods_times: [u8; 24],
}

impl Cmd {
    pub fn new(time_windows: Vec<(u8, u8, u8)>, period_bank_holidays: u8) -> Self {
        Cmd {
            bank_holidays: HashSet::new(),
            period_bank_holidays,
            counters: HashMap::new(),
            periods_times: Self::index_period_times(time_windows),
        }
    }

    // TODO: add a constructor that receives bank holidays and counters.

    // It register the dates to consider them bank holidays for applying the rate of the specified
    // bank holidays period.
    //
    // A duplicated or calling it with already registered dates are ignored, so no error is
    // returned.
    pub fn with_bank_holidays(&mut self, dates: Vec<String>) -> Result<(), String> {
        for mdate in dates {
            let days = utils::parse_date_multiple_days(&mdate)?;
            for day in days {
                self.bank_holidays.insert(day);
            }
        }

        Ok(())
    }

    // Registers the counters of the meter for each period.
    //
    // If a period exists ore than once in `periods`, the last is used. If a period is already
    // registered, it's updated.
    pub fn with_counters(&mut self, periods: Vec<(u8, u64)>) {
        for (p, c) in periods {
            self.counters.insert(p, c);
        }
    }

    fn index_period_times(time_windows: Vec<(u8, u8, u8)>) -> [u8; 24] {
        let mut period_times: [u8; 24] = [0; 24];
        for w in time_windows {
            let (period, mut start, end) = w;
            let mut current_end = if end < start { 24 } else { end };

            loop {
                for t in start..current_end {
                    period_times[t as usize] = period
                }

                if current_end == end {
                    break;
                }

                current_end = end;
                start = 0;
            }
        }

        period_times
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_cmd_index_period_times() {
        let time_windows = vec![
            (1, 10, 14),
            (1, 18, 22),
            (2, 8, 10),
            (2, 14, 18),
            (2, 22, 0),
            (3, 0, 8),
        ];

        let period_times = Cmd::index_period_times(time_windows);
        for i in 0..8 {
            assert_eq!(3, period_times[i], "period in time '{}'", i);
        }

        for i in 8..10 {
            assert_eq!(2, period_times[i], "period in time '{}'", i);
        }

        for i in 10..14 {
            assert_eq!(1, period_times[i], "period in time '{}'", i);
        }

        for i in 14..18 {
            assert_eq!(2, period_times[i], "period in time '{}'", i);
        }

        for i in 18..22 {
            assert_eq!(1, period_times[i], "period in time '{}'", i);
        }

        for i in 22..24 {
            assert_eq!(2, period_times[i], "period in time '{}'", i);
        }
    }

    #[test]
    fn test_cmd_with_bank_holidays() {
        {
            // OK: no duplicates.
            let mut cmd = Cmd::new(vec![], 0);
            cmd.with_bank_holidays(vec!["2022-10-12".to_string(), "2022-12-25,26".to_string()])
                .expect("OK no duplicates");

            assert_eq!(3, cmd.bank_holidays.len(), "Hash set length");
            assert!(
                cmd.bank_holidays.contains("2022-10-12"),
                "Hash set contains 2022-10-12"
            );
            assert!(
                cmd.bank_holidays.contains("2022-12-25"),
                "Hash set contains 2022-12-25"
            );
            assert!(
                cmd.bank_holidays.contains("2022-12-26"),
                "Hash set contains 2022-12-26"
            );
        }

        {
            // OK: with duplicates.
            let mut cmd = Cmd::new(vec![], 0);
            cmd.with_bank_holidays(vec!["2022-12-26".to_string(), "2022-12-25,26".to_string()])
                .expect("OK with duplicates");

            assert_eq!(2, cmd.bank_holidays.len(), "Hash set length");
            assert!(
                cmd.bank_holidays.contains("2022-12-25"),
                "Hash set contains 2022-12-25"
            );
            assert!(
                cmd.bank_holidays.contains("2022-12-26"),
                "Hash set contains 2022-12-26"
            );

            // Calling it again with a dates that already exist ignores them.
            cmd.with_bank_holidays(vec!["2022-12-25".to_string(), "2023-01-06".to_string()])
                .expect("OK with already existing ones");
            assert_eq!(3, cmd.bank_holidays.len(), "Hash set length");
            assert!(
                cmd.bank_holidays.contains("2022-12-25"),
                "Hash set contains 2022-12-25"
            );
            assert!(
                cmd.bank_holidays.contains("2022-12-26"),
                "Hash set contains 2022-12-26"
            );
            assert!(
                cmd.bank_holidays.contains("2023-01-06"),
                "Hash set contains 2023-01-06"
            );
        }

        {
            // Error: invalid date.
            let mut cmd = Cmd::new(vec![], 0);
            cmd.with_bank_holidays(vec!["2022-12-26".to_string(), "2023-02-28,29".to_string()])
                .expect_err("invalid date");
        }
    }

    #[test]
    fn test_cmd_with_counters() {
        {
            // Without duplicates.
            let mut cmd = Cmd::new(vec![], 0);
            cmd.with_counters(vec![(1, 60), (2, 3876), (10, 89)]);

            assert_eq!(3, cmd.counters.len(), "Hash set length");
            assert_eq!(Some(&60), cmd.counters.get(&1), "contains 1");
            assert_eq!(Some(&3876), cmd.counters.get(&2), "contains 2");
            assert_eq!(Some(&89), cmd.counters.get(&10), "contains 10");
        }
        {
            // With duplicates.
            let mut cmd = Cmd::new(vec![], 0);
            cmd.with_counters(vec![(1, 60), (2, 3876), (1, 89)]);

            assert_eq!(2, cmd.counters.len(), "Hash set length");
            assert_eq!(Some(&89), cmd.counters.get(&1), "contains 1");
            assert_eq!(Some(&3876), cmd.counters.get(&2), "contains 2");

            // Calling with existing ones, updates them.
            cmd.with_counters(vec![(10, 60), (2, 30)]);

            assert_eq!(3, cmd.counters.len(), "Hash set length");
            assert_eq!(Some(&89), cmd.counters.get(&1), "contains 1");
            assert_eq!(Some(&30), cmd.counters.get(&2), "contains 2");
            assert_eq!(Some(&60), cmd.counters.get(&10), "contains 10");
        }
    }
}
