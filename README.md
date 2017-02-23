# GridSolver

A dumb little program that fills in empty crossword grids.

### Installation & Usage

You need to have the nightly version of [Rust](https://www.rust-lang.org/en-US/) installed.
Once you do, enable it with `rustup default nightly`.

```sh
git clone https://github.com/wrieger93/gridsolver
cd gridsolver
cargo build --release
cargo run --release
```

It should spit out a completely filled grid.
Solve times can vary wildy; if it's taking more than 10 or 20 seconds, you should restart the program.
To fill a different grid, you have to change the file that's loaded in `src/bin/run.rs`.

### TODO

A lot!
