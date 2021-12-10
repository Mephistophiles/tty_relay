/*
 * Copyright (C) 2020 Maxim Zhukov <mussitantesmortem@gmail.com>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */
/// tty relay manager
use anyhow::Result;
use clap::{crate_authors, crate_version, App, AppSettings, Arg, ArgMatches, ColorChoice};
use clap_generate::generate;
use clap_generate::generators::{Bash, Elvish, Fish, PowerShell, Zsh};
use port::Port;
use std::env;
use std::io;
use std::path::Path;
use std::process;

const APPNAME: &str = "tty_relay";

mod port;

enum Command {
    On,
    Off,
    Toggle,
    Jog,
    TimedOn(u16),
    TimedOff(u16),
    Unknown,
}

fn parse_command(matches: &ArgMatches) -> Command {
    let subcommand = matches.subcommand_name().unwrap();

    if subcommand == "on" {
        Command::On
    } else if subcommand == "off" {
        Command::Off
    } else if subcommand == "toggle" {
        Command::Toggle
    } else if subcommand == "jog" {
        Command::Jog
    } else if let Some(sub_matches) = matches.subcommand_matches("timed_start") {
        let seconds = sub_matches.value_of("seconds").unwrap().parse().unwrap();
        Command::TimedOn(seconds)
    } else if let Some(sub_matches) = matches.subcommand_matches("timed_stop") {
        let seconds = sub_matches.value_of("seconds").unwrap().parse().unwrap();
        Command::TimedOff(seconds)
    } else {
        Command::Unknown
    }
}

fn autocomplete(matches: &ArgMatches, mut app: &mut App) {
    if let Some(generator) = matches.value_of("generator") {
        eprintln!("Generating completion file for {}...", generator);
        match generator {
            "bash" => generate(Bash, &mut app, APPNAME, &mut io::stdout()),
            "elvish" => generate(Elvish, &mut app, APPNAME, &mut io::stdout()),
            "fish" => generate(Fish, &mut app, APPNAME, &mut io::stdout()),
            "powershell" => generate(PowerShell, &mut app, APPNAME, &mut io::stdout()),
            "zsh" => generate(Zsh, &mut app, APPNAME, &mut io::stdout()),
            _ => panic!("Unknown generator"),
        }

        process::exit(0);
    }
}

fn is_number(val: &str) -> Result<(), String> {
    let _: i32 = val
        .parse()
        .map_err(|e: std::num::ParseIntError| e.to_string())?;

    Ok(())
}

fn main() -> Result<()> {
    flexi_logger::Logger::try_with_env()
        .unwrap()
        .start()
        .unwrap();

    let generator_args = || {
        Arg::new("generator")
            .long("generate")
            .value_name("shell")
            .possible_values(&["bash", "elvish", "fish", "powershell", "zsh"])
    };

    let tty_port_arg = || {
        Arg::new("tty port")
            .long("tty")
            .short('t')
            .help("manually select tty port")
            .takes_value(true)
            .validator(|s| {
                let path = Path::new(s);

                if path.exists() {
                    Ok(())
                } else {
                    Err("Invalid path")
                }
            })
    };

    macro_rules! timed_command {
        ($name:expr) => {
            App::new(concat!("timed_", $name))
                .about(concat!($name, " after n seconds"))
                .arg(Arg::new("seconds").required(true).validator(is_number))
        };
    }

    let mut app = App::new(APPNAME)
        .about("tty power management")
        .author(crate_authors!())
        .color(ColorChoice::Auto)
        .setting(AppSettings::ArgRequiredElseHelp)
        .arg(generator_args())
        .arg(tty_port_arg())
        .subcommand(App::new("on").about("enable power"))
        .subcommand(App::new("off").about("disable power"))
        .subcommand(App::new("toggle").about("toggle power"))
        .subcommand(App::new("jog").about("quick toggle power"))
        .subcommand(timed_command!("start"))
        .subcommand(timed_command!("stop"))
        .version(crate_version!());

    let matches = app.clone().get_matches();

    autocomplete(&matches, &mut app);

    let mut port = Port::open(matches.value_of("tty port"))?;

    match parse_command(&matches) {
        Command::On => port.on(),
        Command::Off => port.off(),
        Command::Toggle => port.toggle(),
        Command::Jog => port.jog(),
        Command::TimedOn(secs) => port.timed_on(secs),
        Command::TimedOff(secs) => port.timed_off(secs),
        _ => panic!("unknown command {:?}", matches),
    }
}
