use crate::utils;

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
    /// ^[\d]{4}-[\d]{2}-[\d]{2}(,[\d]{2})*$
    ///
    /// Where the digits are yyyy-mm-dd.
    ///
    /// Set it more than one for setting different bank holidays in different months or in the same
    /// month without using the comma separated days shortcut.
    ///
    /// Examples:
    ///
    /// - 2022-12-25
    ///
    /// - 2022-12-25,26
    #[arg(short = 'd', long, value_parser = parse_date_multiple_days)]
    pub bank_holidays: Option<Vec<String>>,

    /// The meter counters to consider before the first date present in the CSV file.
    ///
    /// These counters are the base to add up the CSV readings according to the time windows. The
    /// missing time windows are considered to be a 0 counter and the ones that don't exist are
    /// ignored.
    /// Currently, the companies use three time windows: p1, p2, p3.
    ///
    /// Set it more than one for setting different periods. If a period is repeated only the last
    /// one is considered.
    ///
    /// Format expressed in a regular expression is: ^p[\d]=[\d]+$
    ///
    /// Examples:
    ///
    /// - p1=97
    ///
    /// - p3=23
    #[arg(short = 'c', long, value_parser = utils::parse_meter_counter)]
    pub base_meter_counter: Option<Vec<(u8, u64)>>,

    /// File path to the e-distribution CSV file
    pub csv_filepath: String,

    /// Time windows contemplate by the electric company with the possibility to apply different
    /// rates on each one.
    // For now the flag isn't exposed but we want the command logic to treat this format to
    // consider the time windows rather than hard coding them.
    // The format could be "p1:10-14" and passed multipel times
    #[arg(skip = [(1, 10, 14), (1, 18, 22), (2, 8, 10), (2, 14, 18), (2, 22, 0), (3, 0, 8)])]
    pub time_windows: Vec<(u8, u8, u8)>,
}

/// Validates a command-line argument that contains a date that can contains more than one day in
/// the specified year and month.
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
fn parse_date_multiple_days(s: &str) -> Result<String, String> {
    utils::parse_date_multiple_days(s)?;
    Ok(s.to_string())
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
    fn test_parse_date_multiple_days() {
        {
            // Valid.
            let input = "2022-12-25";
            let date =
                parse_date_multiple_days(input).expect(&format!(r#""{}" should be valid"#, input));
            assert_eq!(input, date, r#""{}" should only have this date"#, input);

            let input = "2022-12-25,26";
            let date =
                parse_date_multiple_days(input).expect(&format!(r#""{}" should be valid"#, input));
            assert_eq!(input, date, r#""{}" should only have this date"#, input);
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
}
