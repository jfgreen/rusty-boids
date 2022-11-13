use std::{env, fmt, fs::File, io, io::prelude::*, process};

use crate::boids::{SimulationConfig, WindowSize};

use toml;

pub fn build_config() -> Result<SimulationConfig, ConfigError> {
    let mut builder = ConfigBuilder::new();
    let args = env::args();
    let config_path = parse_args(args);
    //TODO: Is this now overcomplicated
    builder.apply(UserSimulationConfig::from_toml_file(&config_path)?);
    Ok(builder.build())
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

fn parse_args(args: impl IntoIterator<Item = String>) -> String {
    let mut args = args.into_iter();
    let exec = args.next();
    match (args.next(), args.next()) {
        (Some(arg), None) => arg,
        _ => {
            let exec = exec.as_deref().unwrap_or("boids");
            println!("Usage: {exec} config-file");
            process::exit(1);
        }
    }
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
        merge(&mut c.boid_size, uc.boid_size);
        if let Some(uc_flock) = uc.flocking {
            merge(&mut c.max_speed, uc_flock.max_speed);
            merge(&mut c.max_force, uc_flock.max_force);
            merge(&mut c.mouse_weight, uc_flock.mouse_weight);
            merge(&mut c.sep_weight, uc_flock.sep_weight);
            merge(&mut c.ali_weight, uc_flock.ali_weight);
            merge(&mut c.coh_weight, uc_flock.coh_weight);
            merge(&mut c.sep_radius, uc_flock.sep_radius);
            merge(&mut c.ali_radius, uc_flock.ali_radius);
            merge(&mut c.coh_radius, uc_flock.coh_radius);
        }
    }

    fn build(self) -> SimulationConfig {
        self.config
    }
}

#[derive(Debug)]
pub enum ConfigError {
    Io(io::Error),
    Toml(toml::de::Error),
}

impl From<io::Error> for ConfigError {
    fn from(err: io::Error) -> ConfigError {
        ConfigError::Io(err)
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
    boid_size: Option<f32>,
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
}
