extern crate clap;
use clap::{Arg, App};

#[macro_use]
extern crate log;
extern crate env_logger;

extern crate gridsolver;
use gridsolver::dict::{RankedDictionary, Dictionary, RankedDict, UnrankedDict};
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
        .arg(Arg::with_name("ranked")
             .short("r")
             .long("ranked")
             .takes_value(false)
             .help("Use this flag if the dictionary is ranked"))
        .arg(Arg::with_name("grid")
             .short("g")
             .long("grid")
             .takes_value(true)
             .default_value("./assets/grid1.txt")
             .help("The file to load the grid from"))
        .get_matches();
    let dict_path = matches.value_of("dict").unwrap();
    let grid_path = matches.value_of("grid").unwrap();
    let dict_ranked = matches.is_present("ranked");

    // load the dictionary and grid
    let grid = Grid::from_file(grid_path).expect("could not load grid");

    // load the appropriate dictionary
    if dict_ranked {
        let dict = <RankedDictionary as RankedDict>::from_file(dict_path).expect("could not load dict");
        let mut solver = GridSolver::new(grid, dict);
        solver.solve();
        println!("{}", solver);
        println!("{}", solver.average_score());
    } else {
        let dict = <Dictionary as UnrankedDict>::from_file(dict_path).expect("could not load dict");
        let mut solver = GridSolver::new(grid, dict);
        solver.solve();
        println!("{}", solver);
    }

}
