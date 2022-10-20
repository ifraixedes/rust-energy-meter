use std::vec::Vec;

use chrono::naive::NaiveDate;

/// Parses a string that contains a date which may contain more than one day in the specified year
/// and month and return the list of dates separately.
///
/// Repeated days don't produce an error.
///
/// Format expressed in a regular expression is:
/// ^[\d]{4}-[\d]{2}-[\d]{2}(,[\d]{2})*$
///
/// Where the digits are yyyy-mm-dd.
///
/// Examples:
///
/// - 2022-12-25
///
/// - 2022-12-25,26
pub fn parse_date_multiple_days(s: &str) -> Result<Vec<String>, String> {
    let mut dates = Vec::new();

    let year_month: String;
    let mut days = s.split(",");
    if let Some(d) = days.next() {
        let parts: Vec<&str> = d.split("-").collect();
        if parts.len() != 3 {
            return Err(format!(
                r#"invalid date "{}", part "{}" contains an invalid number of '-'"#,
                s, d
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
    }

    Ok(dates)
}

/// Validates a command-line argument that contains a meter counter.
///
/// Format expressed in a regular expression is: ^p[\d]=[\d]+$
///
/// Examples:
///
/// - p1=97
///
pub fn parse_meter_counter(s: &str) -> Result<(u8, u64), String> {
    let (name, value) = s
        .split_once("=")
        .ok_or(format!(r#"invalid period "{}", it doesn't have an '='"#, s))?;

    if name.len() != 2 {
        return Err(format!(
            r#"invalid period "{}", it doesn't have a valid period's name "{}". It isn't of the format 'p<single digit number>'."#,
            s, name,
        ));
    }

    if name.get(0..1).unwrap() != "p" {
        return Err(format!(
            r#"invalid period "{}", it doesn't have a valid period's name "{}". It doesn't start with 'p'"#,
            s, name,
        ));
    }

    let pi = name.get(1..2).unwrap().parse::<u8>().map_err(|e| format!(
                r#"invalid period "{}", it doesn't have a valid period's name "{}". It doesn't have a digit after 'p': {}"#,
                s, name, e,
            ))?;

    let pv = value.parse().map_err(|e| format!(
                r#"invalid period "{}", it doesn't have a valid period's value "{}". It isn't an unsigned integer: {}"#,
                s, value, e,
            ))?;

    Ok((pi, pv))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_date_multiple_days() {
        {
            // Valid.
            let input = "2022-12-25";
            let dates =
                parse_date_multiple_days(input).expect(&format!(r#""{}" should be valid"#, input));
            assert_eq!(1, dates.len(), r#""{}" should have 1 date"#, input);
            assert_eq!(input, dates[0], r#""{}" should only have this date"#, input);

            let input = "2022-12-25,26";
            let dates =
                parse_date_multiple_days(input).expect(&format!(r#""{}" should be valid"#, input));
            assert_eq!(2, dates.len(), r#""{}" should have 2 dates"#, input);
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
        }

        {
            // Invalid.
            parse_date_multiple_days("2022-09-31")
                .expect_err("September doesn't have the 31st day");
            parse_date_multiple_days("2022-09-30,31")
                .expect_err("September doesn't have the 31st day");
            parse_date_multiple_days("2022-12-25;26")
                .expect_err("second days separated with semicolon");
        }
    }

    #[test]
    fn test_parse_meter_counter() {
        {
            // Valid.
            let input = "p3=10";
            let counter =
                parse_meter_counter(input).expect(&format!(r#""{}" should be valid"#, input));
            assert_eq!((3, 10), counter);
        }

        {
            // Invalid.
            parse_meter_counter("p1=").expect_err("a period without value");
            parse_meter_counter("p=187").expect_err("invalid period");
            parse_meter_counter("a1=187").expect_err("invalid period");
            parse_meter_counter("p1=18,15").expect_err("invalid value");
        }
    }
}
