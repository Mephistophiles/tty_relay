/// tty relay manager
use clap::{crate_authors, crate_version, App, AppSettings, Arg, ArgMatches};
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
    if matches.is_present("on") {
        return Command::On;
    } else if matches.is_present("off") {
        return Command::Off;
    } else if matches.is_present("toggle") {
        return Command::Toggle;
    } else if matches.is_present("jog") {
        return Command::Jog;
    } else if let Some(sub_matches) = matches.subcommand_matches("timed_start") {
        let seconds = sub_matches.value_of("seconds").unwrap().parse().unwrap();
        return Command::TimedOn(seconds);
    } else if let Some(sub_matches) = matches.subcommand_matches("timed_stop") {
        let seconds = sub_matches.value_of("seconds").unwrap().parse().unwrap();
        return Command::TimedOff(seconds);
    }

    Command::Unknown
}

fn autocomplete(matches: &ArgMatches, mut app: &mut App) {
    if let Some(generator) = matches.value_of("generator") {
        eprintln!("Generating completion file for {}...", generator);
        match generator {
            "bash" => generate::<Bash, _>(&mut app, APPNAME, &mut io::stdout()),
            "elvish" => generate::<Elvish, _>(&mut app, APPNAME, &mut io::stdout()),
            "fish" => generate::<Fish, _>(&mut app, APPNAME, &mut io::stdout()),
            "powershell" => generate::<PowerShell, _>(&mut app, APPNAME, &mut io::stdout()),
            "zsh" => generate::<Zsh, _>(&mut app, APPNAME, &mut io::stdout()),
            _ => panic!("Unknown generator"),
        }

        process::exit(0);
    }
}

fn main() {
    flexi_logger::Logger::with_env().start().unwrap();

    let generator_args = || {
        Arg::new("generator")
            .long("generate")
            .value_name("shell")
            .possible_values(&["bash", "elvish", "fish", "powershell", "zsh"])
    };

    let mut app = App::new(APPNAME)
        .about("tty power management")
        .author(crate_authors!())
        .setting(AppSettings::ColoredHelp)
        .setting(AppSettings::ArgRequiredElseHelp)
        .arg(generator_args())
        .arg(
            Arg::with_name("tty port")
                .long("tty")
                .short('t')
                .about("manually select tty port")
                .takes_value(true)
                .validator(|s| {
                    let path = Path::new(s);

                    if path.exists() {
                        Ok(())
                    } else {
                        Err("Invalid path")
                    }
                }),
        )
        .subcommand(App::new("on").about("enable power"))
        .subcommand(App::new("off").about("disable power"))
        .subcommand(App::new("toggle").about("toggle power"))
        .subcommand(App::new("jog").about("quick toggle power"))
        .subcommand(
            App::new("timed_start")
                .about("start after n seconds")
                .arg(Arg::with_name("seconds").required(true)),
        )
        .subcommand(
            App::new("timed_stop")
                .about("stop after n seconds")
                .arg(Arg::with_name("seconds").required(true)),
        )
        .version(crate_version!());

    let matches = app.clone().get_matches();

    autocomplete(&matches, &mut app);

    let mut port = Port::open(matches.value_of("tty port")).unwrap();

    match parse_command(&matches) {
        Command::On => port.on(),
        Command::Off => port.off(),
        Command::Toggle => port.toggle(),
        Command::Jog => port.jog(),
        Command::TimedOn(secs) => port.timed_on(secs),
        Command::TimedOff(secs) => port.timed_off(secs),
        _ => todo!(),
    }
}
