# tty_relay
TTY Power Management for DC 5V 1 Channel Time Delay Relay CH340 CH340G Smart Programmable USB Time Control Switch Board USB To TTL Module for Arduino PC

# How to install
```shell
cargo install --git https://github.com/Mephistophiles/tty_relay
```

# Connect via NC (Normal Closed) or NO (Normal Open)

By default the tty_relay uses NO connector, but you can select NC via

```shell
cargo install --git https://github.com/Mephistophiles/tty_relay --no-default-features --features="nc-connected"
```

# Examples
Turn on power
```
tty_relay on
```
Turn off power
```
tty_relay off
```

# Available options

```
tty_relay 0.1.0
Maxim Zhukov <mussitantesmortem@gmail.com>
tty power management

USAGE:
    tty_relay [OPTIONS] [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --generate <shell>     [possible values: bash, elvish, fish, powershell, zsh]
    -t, --tty <tty port>      manually select tty port

SUBCOMMANDS:
    help           Prints this message or the help of the given subcommand(s)
    jog            quick toggle power
    off            disable power
    on             enable power
    timed_start    start after n seconds
    timed_stop     stop after n seconds
    toggle         toggle power
```
