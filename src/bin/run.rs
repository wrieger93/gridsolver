#![feature(test)]
extern crate test;

extern crate gridsolver;
use gridsolver::dict::Dictionary;
use gridsolver::grid::*;

fn main() {
    println!("loading dictionary");
    let dict = Dictionary::from_file("assets/ukacd_utf8.txt").expect("could not load dict");
    println!("dictionary loaded");
    println!("loading grid");
    let grid = Grid::from_file("assets/grid1.txt").expect("could not load grid");
    println!("grid loaded\n");

    // keep trying to solve the grid
    let mut solver = GridSolver::new(&grid, &dict);
    while !solver.solve() {
        solver.new_grid(&grid);
    }
    
    // print the filled grid
    println!("{}", solver);
}
