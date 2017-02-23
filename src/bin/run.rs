#![feature(test)]
extern crate test;

extern crate gridsolver;
use gridsolver::dict::Dictionary;
use gridsolver::grid::*;

fn main() {
    let dict = Dictionary::from_file("/usr/share/dict/words").expect("could not load dict");
    let grid = Grid::from_file("assets/grid2.txt").expect("could not load grid");
    let mut solver = GridSolver::new(&grid, &dict);
    loop {
        if solver.solve() {
            println!("{}", solver);
        }
        else {
            println!("retrying");
        }
        solver.new_grid(&grid);
    }
}
