use std::vec::Vec;

use chrono::naive::NaiveDate;
use clap::Parser;

/// Accepted arguments by the command-line application.
#[derive(Parser)]
#[command(author, version, about)]
pub struct App {
    /// List of bank holidays in the period of time present in the CSV file.
    ///
    /// Format expressed in a regular expression is:
    /// ^([\d]{4}-[\d]{2}-[\d]{2}(,[\d]{2})*)(;[\d]{4}-[\d]{2}-[\d]{2}(,[\d]{2})*)*$
    ///
    /// Where the digits are yyyy-mm-dd.
    ///
    /// Examples:
    ///
    /// - 2022-12-25
    ///
    /// - 2022-12-25,26
    ///
    /// - 2022-12-25;2023-01-01
    ///
    /// - 2022-12-25,26;2022-01-01
    ///
    /// - 2022-12-25,26;2022-01-01,06
    #[arg(short = 'd', long, value_parser = dates_list_check)]
    pub bank_holidays: Option<Vec<String>>,

    /// The meter counters to consider before the first date present in the CSV file.
    ///
    /// These counters are the base to add up the CSV readings according to the time windows. The
    /// missing time windows are considered to be a 0 counter and the ones that don't exist are
    /// ignored.
    /// Currently, the companies use three time windows: p1, p2, p3.
    ///
    /// Format expressed in a regular expression is: ^(p[\d]=[\d]+)(,p[\d]=[\d]+)*$
    ///
    /// Examples:
    ///
    /// - p1=97
    ///
    /// - p3=23,p1=7
    #[arg(short = 'c', long, value_parser = meter_counters_check)]
    pub base_meter_counter: Option<Vec<String>>,

    /// File path to the e-distribution CSV file
    pub csv_filepath: String,

    /// Time windows contemplate by the electric company with the possibility to apply different
    /// rates on each one.
    // For now the flag isn't exposed but we want the command logic to treat this format to
    // consider the time windows rather than hard coding them.
    #[arg(skip = "p1:10-14,18-22;p2:08-10,14-18,22-00;p3:00-08")]
    pub time_windows: String,
}

/// Validates a command-line argument that contains a list of dates allowing more than one day in a
/// specific year and month.
///
/// Repeated dates don't produce an error.
///
/// Format expressed in a regular expression is:
/// ^([\d]{4}-[\d]{2}-[\d]{2}(,[\d]{2})*)(;[\d]{4}-[\d]{2}-[\d]{2}(,[\d]{2})*)*$
///
/// Where the digits are yyyy-mm-dd.
///
/// Examples:
///
/// - 2022-12-25
///
/// - 2022-12-25,26
///
/// - 2022-12-25;2023-01-01
///
/// - 2022-12-25,26;2022-01-01
///
/// - 2022-12-25,26;2022-01-01,06
fn dates_list_check(s: &str) -> Result<Vec<String>, String> {
    let mut dates = Vec::new();

    for date_unit in s.split(";") {
        let year_month: String;
        let mut days = date_unit.split(",");
        if let Some(d) = days.next() {
            let parts: Vec<&str> = d.split("-").collect();
            if parts.len() != 3 {
                return Err(format!(
                    r#"invalid date "{}", part "{}" contains an invalid number of '-'"#,
                    date_unit, d
                ));
            }

            year_month = format!("{}-{}", parts[0], parts[1]);

            let date = format!("{}-{}", year_month, parts[2]);
            NaiveDate::parse_from_str(&date, "%Y-%m-%d").map_err(|e| {
                format!(
                    r#"invalid date "{}". It isn't of the format "yyyy-mm-dd" or it isn't a valid date: {}"#,
                    date, e,
                )
            })?;
            dates.push(date);

            for day in days {
                let date = format!("{}-{}", year_month, day);
                NaiveDate::parse_from_str(&date, "%Y-%m-%d").map_err(|_| {
                    format!(
                        r#"invalid date "{}". It isn't of the format "yyyy-mm-dd""#,
                        date
                    )
                })?;
                dates.push(date);
            }
        } else {
            return Err(format!(
                r#"invalid date "{}", it contains an invalid number of ';'"#,
                date_unit
            ));
        }
    }

    Ok(dates)
}

/// Validates a command-line argument that contains a list of meter counters for each time windows.
///
/// Repeated periods don't produce an error.
///
/// Format expressed in a regular expression is: ^(p[\d]=[\d]+)(,p[\d]=[\d]+)*$
///
/// Examples:
///
/// - p1=97
///
/// - p3=23,p1=7
fn meter_counters_check(s: &str) -> Result<Vec<(u8, u64)>, String> {
    let mut periods = Vec::new();

    for period in s.split(",") {
        let (name, value) = period.split_once("=").ok_or(format!(
            r#"invalid period "{}", it doesn't have an '='"#,
            period
        ))?;

        if name.len() != 2 || name.get(0..1).unwrap() != "p" {
            return Err(format!(
                r#"invalid period "{}", it doesn't have a valid period's name "{}". It doesn't start with 'p'"#,
                period, name,
            ));
        }

        let pi = name.get(1..2).unwrap().parse::<u8>().map_err(|e| format!(
                r#"invalid period "{}", it doesn't have a valid period's name "{}". It doesn't have a single digit number after 'p': {}"#,
                period, name, e,
            ))?;

        let pv = value.parse().map_err(|e| format!(
                r#"invalid period "{}", it doesn't have a valid period's value "{}". It isn't an unsigned integer: {}"#,
                period, value, e,
            ))?;

        periods.push((pi, pv));
    }

    Ok(periods)
}

#[cfg(test)]
mod test {
    use super::*;

    use clap::CommandFactory;

    #[test]
    fn verify_cli() {
        App::command().debug_assert();
    }

    #[test]
    fn test_dates_list_check() {
        {
            // Valid
            let input = "2022-12-25";
            let dates = dates_list_check(input).expect(&format!(r#""{}" should be valid"#, input));
            assert_eq!(1, dates.len(), r#""{}" should only have one date"#, input);
            assert_eq!(input, dates[0], r#""{}" should only has this date"#, input);

            let input = "2022-12-25,26";
            let dates = dates_list_check(input).expect(&format!(r#""{}" should be valid"#, input));
            assert_eq!(2, dates.len(), r#""{}" should have two dates"#, input);
            assert_eq!(
                "2022-12-25", dates[0],
                r#""{}" should have this date at index 0"#,
                input
            );
            assert_eq!(
                "2022-12-26", dates[1],
                r#""{}" should have this date at index 1"#,
                input
            );

            let input = "2022-12-25;2023-01-01";
            let dates = dates_list_check(input).expect(&format!(r#""{}" should be valid"#, input));
            assert_eq!(2, dates.len(), r#""{}" should have two dates"#, input);
            assert_eq!(
                "2022-12-25", dates[0],
                r#""{}" should have this date at index 0"#,
                input
            );
            assert_eq!(
                "2023-01-01", dates[1],
                r#""{}" should have this date at index 1"#,
                input
            );

            let input = "2022-12-25,26;2023-01-01";
            let dates = dates_list_check(input).expect(&format!(r#""{}" should be valid"#, input));
            assert_eq!(3, dates.len(), r#""{}" should have three dates"#, input);
            assert_eq!(
                "2022-12-25", dates[0],
                r#""{}" should have this date at index 0"#,
                input
            );
            assert_eq!(
                "2022-12-26", dates[1],
                r#""{}" should have this date at index 1"#,
                input
            );
            assert_eq!(
                "2023-01-01", dates[2],
                r#""{}" should have this date at index 2"#,
                input
            );

            let input = "2022-12-25,26;2023-01-01,06";
            let dates = dates_list_check(input).expect(&format!(r#""{}" should be valid"#, input));
            assert_eq!(4, dates.len(), r#""{}" should have four dates"#, input);
            assert_eq!(
                "2022-12-25", dates[0],
                r#""{}" should have this date at index 0"#,
                input
            );
            assert_eq!(
                "2022-12-26", dates[1],
                r#""{}" should have this date at index 1"#,
                input
            );
            assert_eq!(
                "2023-01-01", dates[2],
                r#""{}" should have this date at index 2"#,
                input
            );
            assert_eq!(
                "2023-01-06", dates[3],
                r#""{}" should have this date at index 3"#,
                input
            );
        }

        {
            // Invalid.
            dates_list_check("2022-09-31").expect_err("September doesn't have the 31st day");
            dates_list_check("2022-09-30,31").expect_err("September doesn't have the 31st day");
            dates_list_check("2022-09-30;2022-09-31")
                .expect_err("September doesn't have the 31st day");
            dates_list_check("2022-12-25;").expect_err("date with semicolon but without a date");
            dates_list_check("2022-12-25,26;").expect_err("date with semicolon but without a date");
            dates_list_check("2022-12-25,26;;2023-01-01")
                .expect_err("date with semicolon but without a date");
            dates_list_check("2022-09-30,")
                .expect_err("date with a command but without a day number");
            dates_list_check("2022-12-25,26;2023-01-01,")
                .expect_err("date with a command but without a day number");
            dates_list_check("2022-12-2,").expect_err("date with an invalid format");
            dates_list_check("2022-12-25,5,").expect_err("date with an invalid format");
            dates_list_check("2022-12-25,26;23-01-01,").expect_err("date with an invalid format");
            dates_list_check("2022-12-25,26;2023-1-01,").expect_err("date with an invalid format");
        }
    }

    #[test]
    fn test_meter_counters_check() {
        {
            // Valid
            let input = "p3=10";
            let counters =
                meter_counters_check(input).expect(&format!(r#""{}" should be valid"#, input));
            assert_eq!(
                1,
                counters.len(),
                r#""{}" should only have one counter"#,
                input
            );
            assert_eq!(
                (3, 10),
                counters[0],
                r#""{}" should have this values at position 0"#,
                input
            );

            let input = "p3=10,p1=9";
            let counters =
                meter_counters_check(input).expect(&format!(r#""{}" should be valid"#, input));
            assert_eq!(2, counters.len(), r#""{}" should have two counters"#, input);
            assert_eq!(
                (3, 10),
                counters[0],
                r#""{}" should have this values at position 0"#,
                input
            );
            assert_eq!(
                (1, 9),
                counters[1],
                r#""{}" should have this values at position 1"#,
                input
            );

            let input = "p3=10,p9=1,p3=98";
            let counters =
                meter_counters_check(input).expect(&format!(r#""{}" should be valid"#, input));
            assert_eq!(
                3,
                counters.len(),
                r#""{}" should have three counters"#,
                input
            );
            assert_eq!(
                (3, 10),
                counters[0],
                r#""{}" should have this values at position 0"#,
                input
            );
            assert_eq!(
                (9, 1),
                counters[1],
                r#""{}" should have this values at position 1"#,
                input
            );
            assert_eq!(
                (3, 98),
                counters[2],
                r#""{}" should have this values at position 2"#,
                input
            );
        }

        {
            // Invalid.
            meter_counters_check("p1=").expect_err("a period without value");
            meter_counters_check("p3=187,p1=").expect_err("a period without value");
            meter_counters_check("p2=,p3=187,p1=").expect_err("periods without values");
            meter_counters_check("p3=187,p2=,p1=87").expect_err("a period without value");
            meter_counters_check("p=187").expect_err("invalid period");
            meter_counters_check("a1=187").expect_err("invalid period");
            meter_counters_check("p1=18,p10=20").expect_err("invalid period");
            meter_counters_check("p1=18,p99=20,p4=25").expect_err("invalid period");
            meter_counters_check("p1=18,p4=20,").expect_err("comma without a following period");
            meter_counters_check("p1=18,,p4=25").expect_err("comma without a following period");
            meter_counters_check("p1=18,15").expect_err("invalid value");
            meter_counters_check("p2=98,p1=18.15").expect_err("invalid value");
            meter_counters_check("p2=98,p1=kbudhy,p3=18").expect_err("invalid value");
        }
    }
}
