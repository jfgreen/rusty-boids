use std::fmt;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::process;

use boids::{SimulationConfig, WindowSize};

use clap::ErrorKind::{HelpDisplayed, VersionDisplayed};
use clap::{self, App, Arg, ArgMatches};
use toml;

const CONFIG_ARG: &str = "config";
const WINDOW_SIZE_ARG: &str = "size";
const FULLSCREEN_ARG: &str = "fullscreen";
const BOID_COUNT_ARG: &str = "boids";
const DEBUG_ARG: &str = "debug";

pub fn build_config() -> Result<SimulationConfig, ConfigError> {
    let mut builder = ConfigBuilder::new();

    let cli_args = parse_cli_args()?;

    if let Some(path) = cli_args.value_of(CONFIG_ARG) {
        builder.apply(UserSimulationConfig::from_toml_file(path)?);
    }
    builder.apply(UserSimulationConfig::from_cli_args(&cli_args)?);

    Ok(builder.build())
}

struct ConfigBuilder {
    config: SimulationConfig,
}

impl ConfigBuilder {
    fn new() -> Self {
        ConfigBuilder {
            config: SimulationConfig::default(),
        }
    }

    fn apply(&mut self, uc: UserSimulationConfig) {
        let c = &mut self.config;
        merge(&mut c.boid_count, uc.boid_count);
        merge(&mut c.debug, uc.debug);
        merge(&mut c.window_size, window_size(uc.window));
        if let Some(uc_flock) = uc.flocking {
            let c_flock = &mut c.flock_conf;
            merge(&mut c_flock.max_speed, uc_flock.max_speed);
            merge(&mut c_flock.max_force, uc_flock.max_force);
            merge(&mut c_flock.mouse_weight, uc_flock.mouse_weight);
            merge(&mut c_flock.sep_weight, uc_flock.sep_weight);
            merge(&mut c_flock.ali_weight, uc_flock.ali_weight);
            merge(&mut c_flock.coh_weight, uc_flock.coh_weight);
            merge(&mut c_flock.sep_radius, uc_flock.sep_radius);
            merge(&mut c_flock.ali_radius, uc_flock.ali_radius);
            merge(&mut c_flock.coh_radius, uc_flock.coh_radius);
        }
    }

    fn build(self) -> SimulationConfig {
        self.config
    }
}

fn merge<T>(existing: &mut T, candidate: Option<T>) {
    if let Some(v) = candidate {
        *existing = v;
    }
}

fn window_size(window_conf: Option<UserWindowConfig>) -> Option<WindowSize> {
    match window_conf {
        Some(UserWindowConfig {
            fullscreen: Some(true),
            ..
        }) => Some(WindowSize::Fullscreen),
        Some(UserWindowConfig {
            size: Some(dims), ..
        }) => Some(WindowSize::Dimensions(dims)),
        _ => None,
    }
}

fn parse_cli_args() -> Result<ArgMatches<'static>, clap::Error> {
    let args = App::new("boid-simulator")
        .version("0.1")
        .author("James Green")
        .about("Simulates flocking behaviour of birds")
        .arg(
            Arg::with_name(CONFIG_ARG)
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Sets the config file to read simulation parameters from"),
        )
        .arg(
            Arg::with_name(WINDOW_SIZE_ARG)
                .short("s")
                .long("size")
                .value_names(&["width", "height"])
                .use_delimiter(true)
                .help("Sets the simultion window to specified width & height"),
        )
        .arg(
            Arg::with_name(FULLSCREEN_ARG)
                .short("f")
                .long("fullscreen")
                .help("Display fullscreen (overrides size argument)")
                .conflicts_with("size"),
        )
        .arg(
            Arg::with_name(BOID_COUNT_ARG)
                .short("b")
                .long("boid-count")
                .takes_value(true)
                .help("Sets the number of boids to simulate"),
        )
        .arg(
            Arg::with_name(DEBUG_ARG)
                .short("d")
                .long("debug")
                .help("print opengl debug information"),
        )
        .get_matches_safe();

    if let Err(ref err) = args {
        if err.kind == VersionDisplayed || err.kind == HelpDisplayed {
            err.exit(); // Exit and print help message
        }
    }
    args
}

#[derive(Debug)]
pub enum ConfigError {
    Io(io::Error),
    Clap(clap::Error),
    Toml(toml::de::Error),
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

impl From<toml::de::Error> for ConfigError {
    fn from(err: toml::de::Error) -> ConfigError {
        ConfigError::Toml(err)
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ConfigError::Io(ref err) => write!(f, "Could not read config: {}", err),
            ConfigError::Clap(ref err) => write!(f, "{}", err),
            ConfigError::Toml(ref err) => write!(f, "Could not parse toml: {}", err),
        }
    }
}

impl ConfigError {
    pub fn exit(&self) -> ! {
        println!("{}", self);
        process::exit(1);
    }
}

#[derive(Deserialize, Default)]
struct UserSimulationConfig {
    boid_count: Option<u32>,
    debug: Option<bool>,
    window: Option<UserWindowConfig>,
    flocking: Option<UserFlockingConfig>,
}

#[derive(Copy, Clone, Deserialize, Default)]
struct UserWindowConfig {
    size: Option<(u32, u32)>,
    fullscreen: Option<bool>,
}

//TODO: Use rename annoations to make these nicer for the user
#[derive(Copy, Clone, Deserialize, Default)]
struct UserFlockingConfig {
    max_speed: Option<f32>,
    max_force: Option<f32>,
    mouse_weight: Option<f32>,
    sep_weight: Option<f32>,
    ali_weight: Option<f32>,
    coh_weight: Option<f32>,
    sep_radius: Option<f32>,
    ali_radius: Option<f32>,
    coh_radius: Option<f32>,
}

impl UserSimulationConfig {
    fn from_toml_file(path: &str) -> Result<Self, ConfigError> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(toml::from_str(&contents)?)
    }

    fn from_cli_args(args: &ArgMatches<'static>) -> Result<Self, ConfigError> {
        let mut user_conf = UserSimulationConfig::default();
        let mut window_conf = UserWindowConfig::default();

        if args.is_present(BOID_COUNT_ARG) {
            user_conf.boid_count = Some(value_t!(args, BOID_COUNT_ARG, u32)?);
        };

        if args.is_present(DEBUG_ARG) {
            user_conf.debug = Some(true);
        };

        if args.is_present(FULLSCREEN_ARG) {
            window_conf.fullscreen = Some(true);
        };

        if args.is_present(WINDOW_SIZE_ARG) {
            let size = values_t!(args, WINDOW_SIZE_ARG, u32)?;
            window_conf.size = Some((size[0], size[1]));
        };

        user_conf.window = Some(window_conf);
        Ok(user_conf)
    }
}
