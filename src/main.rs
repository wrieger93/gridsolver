#![feature(test)]
extern crate test;

extern crate clap;
use clap::{Arg, App};

extern crate gridsolver;
use gridsolver::dict::Dictionary;
use gridsolver::grid::*;

fn main() {
    // get the command line arguments
    let matches = App::new("GridSolver")
        .about("Fills in empty crossword grids with dictionary words")
        .arg(Arg::with_name("dict")
             .short("d")
             .long("dict")
             .takes_value(true)
             .default_value("./assets/ukacd_utf8.txt")
             .help("The file to load the dictionary from"))
        .arg(Arg::with_name("grid")
             .short("g")
             .long("grid")
             .takes_value(true)
             .default_value("./assets/grid1.txt")
             .help("The file to load the grid from"))
        .get_matches();
    let dict_path = matches.value_of("dict").unwrap();
    let grid_path = matches.value_of("grid").unwrap();

    // load the dictionary and grid
    let dict = Dictionary::from_file(dict_path).expect("could not load dict");
    let grid = Grid::from_file(grid_path).expect("could not load grid");

    // keep trying to solve the grid
    let mut solver = GridSolver::new(&grid, &dict);
    while !solver.solve() {
        solver.new_grid(&grid);
    }

    // print the filled grid
    println!("{}", solver);
}
