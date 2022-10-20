use clap::Parser;

mod cli;
mod cmd;
mod utils;

fn main() {
    let cli = cli::App::parse();

    println!("{:?}", cli.bank_holidays.as_deref());
    println!("{:?}", cli.base_meter_counter.as_deref());
    println!("{:?}", cli.time_windows);
}
