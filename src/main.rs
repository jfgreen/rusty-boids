use aproxiflock::boids::run_simulation;
use aproxiflock::config::build_config;

fn main() {
    let config = build_config().unwrap_or_else(|err| {
        println!("{}", "Failure building configuration:");
        err.exit()
    });

    run_simulation(config).unwrap_or_else(|err| {
        println!("{}", "Failure running simulation");
        err.exit()
    });
}
