use std::io;
use std::io::prelude::*;
use std::fs::File;
use std::fmt;
use std::process;

use boids::{SimulationConfig, WindowSize};

use clap::{self, App, Arg, ArgMatches};
use clap::ErrorKind::{VersionDisplayed, HelpDisplayed};

const CONFIG_ARG: &str = "config";
const WINDOW_SIZE_ARG: &str = "size";
const FULLSCREEN_ARG: &str = "fullscreen";
const BOID_COUNT_ARG: &str = "boids";
const DEBUG_ARG: &str = "debug";

pub fn build_config() -> Result<SimulationConfig, ConfigError> {
    let mut builder = ConfigBuilder::new();

    let cli_args = parse_cli_args()?;

    if let Some(path) = cli_args.value_of(CONFIG_ARG) {
        builder.apply(&UserConfig::from_toml_file(path)?);
    }
    builder.apply(&UserConfig::from_cli_args(&cli_args)?);

    Ok(builder.build())

}

struct ConfigBuilder {
    config: SimulationConfig,
}

impl ConfigBuilder {

    fn new() -> Self {
        ConfigBuilder{ config: SimulationConfig::default() }
    }

    fn apply(&mut self, uc: &UserConfig) {
        let c = &mut self.config;
        merge(&mut c.boid_count,  &uc.boid_count);
        merge(&mut c.window_size, &uc.window_size);
        merge(&mut c.debug,       &uc.debug);
    }

    fn build(self) -> SimulationConfig {
        self.config
    }

}

fn merge<T>(existing: &mut T, candidate: &Option<T>) where T: Copy {
    *existing = candidate.unwrap_or(*existing);
}


//TODO: Would be cool if there was an arg to print / generate an example config file
fn parse_cli_args() -> Result<ArgMatches<'static>, clap::Error> {
    let args = App::new("boid-simulator")
        .version("0.1")
        .author("James Green")
        .about("Simulates flocking behaviour of birds")
        .arg(Arg::with_name(CONFIG_ARG)
             .short("c")
             .long("config")
             .value_name("FILE")
             .help("Sets the config file to read simulation parameters from"))
        .arg(Arg::with_name(WINDOW_SIZE_ARG)
             .short("s")
             .long("size")
             .value_names(&["width", "height"])
             .use_delimiter(true)
             .help("Sets the simultion window to specified width & height"))
        .arg(Arg::with_name(FULLSCREEN_ARG)
             .short("f")
             .long("fullscreen")
             .help("Display fullscreen (overrides size argument)")
             .conflicts_with("size"))
        .arg(Arg::with_name(BOID_COUNT_ARG)
             .short("b")
             .long("boid-count")
             .takes_value(true)
             .help("Sets the number of boids to simulate"))
        .arg(Arg::with_name(DEBUG_ARG)
             .short("d")
             .long("debug")
             .help("print opengl debug information"))
        .get_matches_safe();

    match args {
        Err(ref err) if (err.kind == VersionDisplayed) |
                        (err.kind == HelpDisplayed) => err.exit(),
        _ => args
    }
}


#[derive(Debug)]
pub enum ConfigError {
    Io(io::Error),
    Clap(clap::Error),
}

impl From<io::Error> for ConfigError {
   fn from(err: io::Error) -> ConfigError {
       ConfigError::Io(err)
   }
}

impl From<clap::Error> for ConfigError {
    fn from(err: clap::Error) -> ConfigError {
        ConfigError::Clap(err)
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ConfigError::Io(ref err) =>
                write!(f, "Could not read config:\n\n{}", err),
            ConfigError::Clap(ref err) =>
                write!(f, "Could not parse arguments:\n\n{}", err),
        }
    }
}

impl ConfigError {
   pub fn exit(&self) -> ! {
        println!("{}", self);
        process::exit(1);
   }
}


struct UserConfig {
    boid_count: Option<u32>,
    window_size: Option<WindowSize>,
    debug: Option<bool>,
}

impl UserConfig {
    fn from_toml_file(path: &str) -> Result<Self, ConfigError> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        println!("{}", contents);
        Ok(UserConfig{boid_count:Some(100), ..Default::default()})
    }

    fn from_cli_args(args: &ArgMatches<'static>) -> Result<Self, ConfigError> {

        let window_size = if args.is_present(FULLSCREEN_ARG) {
            Some(WindowSize::Fullscreen)
        } else if args.is_present(WINDOW_SIZE_ARG) {
            let size = values_t!(args, WINDOW_SIZE_ARG, u32)?;
            Some(WindowSize::Dimensions((size[0], size[1])))
        } else {
            None
        };


        let boid_count = if args.is_present(BOID_COUNT_ARG) {
            Some(value_t!(args, BOID_COUNT_ARG, u32)?)
        } else {
            None
        };

        let debug = if args.is_present(DEBUG_ARG) {
            Some(true)
        } else {
            None
        };

        Ok(UserConfig{
            boid_count,
            window_size,
            debug,
            ..Default::default()
        })
    }
}

impl Default for UserConfig {
    fn default() -> UserConfig {
        UserConfig {
            boid_count: None,
            window_size: None,
            debug: None,
        }
    }
}
