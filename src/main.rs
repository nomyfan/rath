extern crate clap;
extern crate sysfs_gpio;
extern crate xshell;

use clap::{App, Arg};
use std::thread::sleep;
use std::time::Duration;
use sysfs_gpio::{Direction, Pin};
use xshell::cmd;

fn main() {
    let mut app = App::new("Raspberry fan controller")
        .version("0.1.0")
        .arg(
            Arg::with_name("temp")
                .short("t")
                .long("temp")
                .value_name("OVER_TEMP")
                .help("Set the over temperature as trigger to run the fan")
                .default_value("43.0"),
        )
        .arg(
            Arg::with_name("pin")
                .short("p")
                .long("pin")
                .value_name("PIN")
                .help("Set gpio pin")
                .default_value("14"),
        );
    let matches = app.clone().get_matches();

    if matches.is_present("h") {
        match app.print_help() {
            Ok(_) => std::process::exit(0),
            Err(_) => std::process::exit(1),
        }
    }

    let over_temp = matches
        .value_of("temp")
        .map(|x| x.parse::<f32>().unwrap_or(43.0))
        .unwrap_or(43.0);
    println!("Over temp: {}°C", over_temp);

    // see: https://pinout.xyz/
    let pin = matches
        .value_of("pin")
        .map(|x| x.parse::<u64>().unwrap_or(14))
        .unwrap_or(14);
    println!("Pin: {}", pin);

    let pin = Pin::new(pin);
    pin.with_exported(|| loop {
        pin.set_direction(Direction::Out)?;

        let high = 1u8;
        let low = 0u8;

        let err_next_duration = Duration::from_secs(2);
        let ok_next_duration = Duration::from_secs(5);
        let fan_ruuning_duration = Duration::from_secs(15);

        loop {
            let temp = read_current_temp().unwrap_or(over_temp);
            println!("Current temp: {}°C", temp);

            let next_duration: Duration;
            if temp > over_temp {
                next_duration = if pin.set_value(high).is_ok() {
                    // keep fan running for seconds
                    sleep(fan_ruuning_duration);
                    ok_next_duration
                } else {
                    err_next_duration
                }
            } else {
                next_duration = if pin.set_value(low).is_ok() {
                    ok_next_duration
                } else {
                    err_next_duration
                };
            }
            sleep(next_duration);
        }
    })
    .unwrap();
}

fn read_current_temp() -> Option<f32> {
    let output = cmd!("vcgencmd measure_temp").read();
    match output {
        Ok(output) => output[5..output.len() - 2].parse::<f32>().ok(),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_read_current_temp() {
        let temp = crate::read_current_temp();
        assert!(temp.is_some());
    }
}
