dead_time
============

This is an simulation of a particle physics collider experimental trigger to calculate the dead time for different derandomazing buffer sizes. The simulation is performed using a Monte Carlo approach. Several
buffer sizes are tested in parallel using threads.

## Model
For a bunch crossing of 40 MHz and a trigger rate of 40 kHz a derandomizing buffer of a certain size is assumed. The trigger needs five bunch crossings time to read out the detector data, afterwards the data
arrives immediately at the buffer. A buffer read of the full detector data from the buffer lasts 424 bunch crossings.

Dead time is assumed to be all triggered events that are not recorded.

## Installation
Cloning the project and running `cargo build --release` in the directory is sufficient if rust is installed. For installing rust please see [rustup](https://rustup.rs/).

## Usage
It is strongly recommend to build with the `--release` flag to use compiler optimizations. Differences in runtimes are huge.

When running **dead_time** you can parse two parameters to the program to configure the buffer sizes. Both are integer values that determine the ranges of buffer sizes to try. First comes the maximum size,
then the minimum size. Defaults are given to be 

  * `15` for maximum size
  * `0` for minimum size
  
The default loop would therefore run over 0 - 15 buffer sizes.

For example running with defaults would be:

```
cargo run --release
```

while running from 0-10 is achieved by

```
cargo run --release 10
```

and running with buffer sizes 5-10 by

```
cargo run --release 10 5
```

Of course one can also simply run the binary itself after compiling.

## License
This project is licensed under the terms of the GPL v3 or any later version (**GPL-3.0-or-later**).

dead_time Copyright (C) 2018 **Fabian A.J. Thiele**, <fabian.thiele@cern.ch>
