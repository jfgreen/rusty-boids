# Rust Boids

This is a 2D boid simulator written in Rust. 🕊

To run with a given number of boids:

`cargo run --release -- -b 40000`

Aims:

- Fast, CPU based simulation.
- Support as many boids as possible.
- Render at 60fps on commodity hardware.

This is achieved by using:

- Approximate "neighbour grid" data structure.
- Lookup table based FOV culling
- (In progress) Parameters that expose performance/accuracy tradeoffs to the user.

It also shows how the `glutin`, `gl` and `cgmath` crates can be used together to build a simulation.
In particular, it demonstrates the boilerplate needed to do useful work with OpenGL.

A big list of things worth doing or looking into  are listed in [TODO.md](TODO.md).

## Configuring

The simulation parameters can be set via a toml configuration file.

```
`cargo run --release -- -c simulation.toml`
```

See `example-config.toml` for an explination of the different parameters.
