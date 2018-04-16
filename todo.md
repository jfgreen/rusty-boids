#Future work

## Simulation

### Implementation improvements

- Refactor simulation, current structs don't feel right.
- Use of "margins" in wrap around code is currently a bit of a hack. 
- Make simulation frame independent (fix your time step article is great)
- Zero copy boids positions into the GPU
- Use sentinel values in spatial grid to allow exactly the number of requested boids.
- Speed up computation with a parallel collection library like Rayon.
- Dynamically select correct shell gap starting size.
- Use a fast number swap trick when sorting grid.
- Lose unnecessary use of Box, e.g in neighbourhood lookup.
- Sort the neighbourhood lookup arrays into memory access pattern order.

### Ideas

#### Up next

- Allow simulation parameters to be supplied via config file
- Option to support cursor interaction only when pressing down the mouse button.

#### Maybe one day

- Support automatically reloading config file when it changes.
- Dynamically calculate pleasing default parameters based on window size and resolution.
- Further explore the feel of the simulation.
    * Different sized neighbourhood lookup table patterns.
    * Can we detect how busy the neighbourhood is and use it to scale repulsion,
      based only on some immediate/sampled neighbours positions?
      - Could such a "panic factor" overcome MAX_FORCE? Have a dynamic max force?
      - You could infer than an area is busy from extreme closeness
      - Maybe we can take a cue from reynolds subsumption architecture?
        Disable one behaviour in favour of another?
    * Allow user to tradeoff between perf and accuracy.
    * Throw in randomness or bias to partial sorting algorithm
    * When things are busy/crowded/"angry":
        - Use a dynamic neighbourhood range (don't need big range for "calm" flock)
        - Sample neighbourhood
        - Add a random "panic" force 

## Renderer

### Implementation improvements

- Handle resizing of screen.
- Use `hidpi_factor` to scale `gl_PointSize`.
- Run gl program and shader cleanup on exit

### Ideas

#### Up Next

- Offer more than one renderers/shader.
- Pretty colours!

#### Maybe one day

- Render velocity somehow?
- Support switching between different renderers/shaders at run time.
- Allow running at different resolutions.


## Fps Counter

- Rethink how caching works, maybe this doesn't live in `fps` module. 
- Consider building an ring buffer of `Instant` instead of `Duration`
