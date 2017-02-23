use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fs::File;
use std::io::prelude::*;
use std::io;
use std::iter::Iterator;
use std::path::Path;
use std::convert::TryFrom;

use rand::{thread_rng, Rng};

use basic_types::*;
use dict::Dictionary;

// Grid

#[derive(Clone, Debug)]
pub struct Grid {
    cells: Vec<Cell>,
    entries: HashMap<EntryIndex, Vec<GridCoord>>,
    perpendicular_entries: HashMap<EntryIndex, Vec<EntryIndex>>,
    width: usize,
    height: usize,
}

impl Grid {
    pub fn new(width: usize, height: usize) -> Option<Grid> {
        if width == 0 || height == 0 {
            return None;
        }
        let mut grid = Grid {
            cells: vec![Cell::White(None); width * height],
            entries: HashMap::new(),
            perpendicular_entries: HashMap::new(),
            width: width,
            height: height,
        };
        grid.rebuild();
        Some(grid)
    }

    pub fn from_cells(cells: &[Cell], width: usize, height: usize) -> Option<Grid> {
        if width == 0 || height == 0 {
            return None;
        }

        let mut cells_vec: Vec<Cell> = cells.into_iter().cloned().collect();
        cells_vec.resize(width * height, Cell::White(None));

        let mut grid = Grid {
            cells: cells_vec,
            entries: HashMap::new(),
            perpendicular_entries: HashMap::new(),
            width: width,
            height: height,
        };
        grid.rebuild();
        Some(grid)
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> io::Result<Grid> {
        // read the file
        let file = try!(File::open(path));
        let mut reader = io::BufReader::new(file);
        let mut entire: String = String::new();
        try!(reader.read_to_string(&mut entire));

        let dimensions: Vec<usize> = entire.chars()
            .take_while(|c| *c != '\n')
            .collect::<String>()
            .split(',')
            .map(|s| s.trim().parse().unwrap())
            .collect();
        let width = dimensions[0];
        let height = dimensions[1];

        let cells: Vec<Cell> = entire.chars()
            .skip_while(|c| *c != '\n')
            .skip(1)
            .filter_map(|c| match c {
                '.' => Some(Cell::White(None)),
                '#' => Some(Cell::Black),
                _ => None,
            })
            .collect();

        let mut grid = Grid {
            cells: cells,
            entries: HashMap::new(),
            perpendicular_entries: HashMap::new(),
            width: width,
            height: height,
        };
        grid.rebuild();
        Ok(grid)
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn get_cell(&self, coord: GridCoord) -> Option<Cell> {
        self.cells.get(self.coord_to_index(coord)).cloned()
    }

    pub fn set_cell(&mut self, coord: GridCoord, val: Cell) {
        let i = self.coord_to_index(coord);
        if let Some(cell) = self.cells.get_mut(i) {
            *cell = val;
        }
    }

    pub fn get_entry_coords(&self, index: EntryIndex) -> Option<Vec<GridCoord>> {
        self.entries.get(&index).cloned()
    }

    pub fn get_entry(&self, index: EntryIndex) -> Option<Entry> {
        self.entries.get(&index)
            .map(|coords| {
                let letter_vec: Vec<Option<Letter>> = coords.iter()
                    .map(|coord| {
                        let cell = self.get_cell(*coord);
                        match cell {
                            Some(Cell::White(c)) => c,
                            _ => unreachable!(),
                        }
                    })
                    .collect();
                Entry::new(letter_vec)
            })
    }

    pub fn set_entry(&mut self, index: EntryIndex, entry: &Entry) {
        if self.entries.contains_key(&index) {
            let coords: Vec<GridCoord> = self.entries[&index].clone();
            for (coord, letter) in coords.into_iter().zip(entry.letters.iter()) { 
                let new_cell = Cell::White(*letter);
                self.set_cell(coord, new_cell);
            }
        }
    }

    pub fn fill_entry(&mut self, index: EntryIndex, word: &Word) {
        if self.entries.contains_key(&index) {
            let coords: Vec<GridCoord> = self.entries[&index].clone();
            let letters: Vec<Letter> = word.letters.clone();
            for (coord, letter) in coords.into_iter().zip(letters.into_iter()) { 
                let new_cell = Cell::White(Some(letter));
                self.set_cell(coord, new_cell);
            }
        }
    }

    pub fn clear_entry(&mut self, index: EntryIndex) {
        if self.entries.contains_key(&index) {
            let coords: Vec<GridCoord> = self.entries[&index].clone();
            for coord in coords { 
                self.set_cell(coord, Cell::White(None));
            }
        }
    }

    pub fn entries_perp_to(&self, index: EntryIndex) -> Vec<EntryIndex> {
        self.perpendicular_entries[&index].clone()
    }

    pub fn is_entry_filled(&self, index: EntryIndex) -> bool {
        match self.get_entry(index) {
            Some(entry) => {
                for x in &entry.letters {
                    if x.is_none() {
                        return false;
                    }
                }
                true
            },
            None => false,
        }
    }

    pub fn is_filled(&self) -> bool {
        self.cells.iter().all(|cell| cell.is_black() || cell.is_filled())
    }

    pub fn entry_indices(&self) -> Vec<EntryIndex> {
        self.entries.keys().cloned().collect()
    }

    pub fn entries(&self) -> Vec<Entry> {
        self.entries.keys()
            .map(|i| self.get_entry(*i).unwrap())
            .collect()
    }


    pub fn rows(&self) -> Vec<Vec<Cell>> {
        (0..self.height)
            .map(|row| {
                (0..self.width)
                    .map(|col| self.get_cell((row, col).into()).unwrap())
                    .collect()
            })
            .collect()
    }

    pub fn cols(&self) -> Vec<Vec<Cell>> {
        (0..self.width)
            .map(|col| {
                (0..self.height)
                    .map(|row| self.get_cell((row, col).into()).unwrap())
                    .collect()
            })
            .collect()
    }

    #[inline]
    fn coord_to_index(&self, coord: GridCoord) -> usize {
        coord.row * self.width + coord.col
    }

    fn across_entry_coords(&self) -> Vec<Vec<GridCoord>> {
        let mut entry_coords_vec = vec![];
        for (row, row_vec) in self.rows().iter().enumerate() {
            let mut in_entry = false;
            let mut entry_coords = vec![];
            for (col, &cell) in row_vec.iter().enumerate() {
                match cell {
                    Cell::White(_) => {
                        in_entry = true;
                        entry_coords.push((row, col).into());
                    },
                    Cell::Black => {
                        if in_entry {
                            in_entry = false;
                            if entry_coords.len() >= 3 {
                                entry_coords_vec.push(entry_coords.clone());
                            }
                            entry_coords.clear();
                        }
                    }
                }
            }
            if in_entry && entry_coords.len() >= 3 {
                entry_coords_vec.push(entry_coords.clone());
            }
        }
        entry_coords_vec.sort();
        entry_coords_vec
    }

    fn down_entry_coords(&self) -> Vec<Vec<GridCoord>> {
        let mut entry_coords_vec = vec![];
        for (col, col_vec) in self.cols().iter().enumerate() {
            let mut in_entry = false;
            let mut entry_coords = vec![];
            for (row, &cell) in col_vec.iter().enumerate() {
                match cell {
                    Cell::White(_) => {
                        in_entry = true;
                        entry_coords.push((row, col).into());
                    },
                    Cell::Black => {
                        if in_entry {
                            in_entry = false;
                            if entry_coords.len() >= 3 {
                                entry_coords_vec.push(entry_coords.clone());
                            }
                            entry_coords.clear();
                        }
                    }
                }
            }
            if in_entry && entry_coords.len() >= 3 {
                entry_coords_vec.push(entry_coords.clone());
            }
        }
        entry_coords_vec.sort();
        entry_coords_vec
    }

    fn rebuild(&mut self) {
        self.entries.clear();
        let across = self.across_entry_coords();
        let down = self.down_entry_coords();
        let mut entry_counter = 1;
        for row in 0..self.height {
            for col in 0..self.width {
                let mut added_entry = false;
                let coord: GridCoord = (row, col).into();
                // add an entry if its first coord matches the current coord
                // at most one entry_coords will match for each of across and down
                for entry_coords in &across {
                    if entry_coords[0] == coord {
                        added_entry = true;
                        let entry_num = EntryIndex::try_from((entry_counter, EntryDir::Across)).unwrap();
                        self.entries.insert(entry_num, entry_coords.clone());
                        break;
                    }
                }
                for entry_coords in &down {
                    if entry_coords[0] == coord {
                        added_entry = true;
                        let entry_num = EntryIndex::try_from((entry_counter, EntryDir::Down)).unwrap();
                        self.entries.insert(entry_num, entry_coords.clone());
                        break;
                    }
                }
                if added_entry {
                    entry_counter += 1;
                }
            }
        }

        self.perpendicular_entries.clear();
        for entry_num in self.entry_indices() {
            let coords = self.entries[&entry_num].clone();
            let mut perpendiculars: Vec<EntryIndex> = vec![];
            for coord in coords {
                for other_entry_num in self.entry_indices() {
                    if entry_num == other_entry_num {
                        continue;
                    }
                    let other_coords = self.entries[&other_entry_num].clone();
                    if other_coords.contains(&coord) {
                        perpendiculars.push(other_entry_num);
                    }
                }
            }
            self.perpendicular_entries.insert(entry_num, perpendiculars);
        }
    }
}

impl fmt::Display for Grid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for row in 0..self.height {
            for col in 0..self.width {
                let cell = self.get_cell((row, col).into()).unwrap();
                match cell {
                    Cell::Black => {
                        write!(f, "\u{2588}")?;
                    }
                    Cell::White(None) => {
                        write!(f, " ")?;
                    }
                    Cell::White(Some(c)) => {
                        write!(f, "{}", c)?;
                    }
                }
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct GridSolver {
    grid: Grid,
    dict: Dictionary,
    added_words: HashSet<Word>,
    unfilled_entries: HashSet<EntryIndex>,
    possible_fills: HashMap<EntryIndex, Vec<Word>>,
    changes: Vec<(EntryIndex, Entry)>,
}

impl GridSolver {
    pub fn new(grid: &Grid, dict: &Dictionary) -> GridSolver {
        let mut solver = GridSolver {
            grid: grid.clone(),
            dict: dict.clone(),
            added_words: HashSet::new(),
            unfilled_entries: HashSet::new(),
            possible_fills: HashMap::new(),
            changes: vec![],
        };

        for index in solver.grid.entry_indices() {
            solver.update_possible_fills(index);
            solver.unfilled_entries.insert(index);
        }

        solver
    }

    fn update_possible_fills(&mut self, index: EntryIndex) {
        let opt = self.grid.get_entry(index);
        match opt {
            Some(entry) => {
                let pattern = Pattern::new(&entry.letters);
                let fills = self.dict.lookup(&pattern);
                self.possible_fills.insert(index, fills);
            }
            _ => {},
        };
    }

    fn fill(&mut self, index: EntryIndex, word: &Word) {
        self.changes.push((index, self.grid.get_entry(index).unwrap()));
        self.grid.fill_entry(index, word);
        self.possible_fills.remove(&index);
        self.unfilled_entries.remove(&index);
        for perp in self.grid.entries_perp_to(index) {
            self.update_possible_fills(perp);
        }
    }

    fn undo_last_fill(&mut self) {
        if self.changes.is_empty() {
            return;
        }
        let (index, prev_entry) = self.changes.pop().unwrap();
        self.grid.set_entry(index, &prev_entry);
        self.possible_fills.insert(index, vec![]);
        self.unfilled_entries.insert(index);
        self.update_possible_fills(index);
        for perp in self.grid.entries_perp_to(index) {
            self.update_possible_fills(perp);
        }
    }

    fn score_fill(&mut self, index: EntryIndex, word: &Word) -> u64 {
        self.fill(index, word);
        let mut score = 0u64;
        for perp in &self.grid.entries_perp_to(index) {
            score *= self.possible_fills[perp].len() as u64;
        }
        self.undo_last_fill();
        score
    }

    pub fn solve(&mut self) -> bool {
        // println!("{}", self.grid);
        if self.unfilled_entries.is_empty() {
            return true;
        }

        let most_constrained = self.unfilled_entries.iter()
            .min_by_key(|index| self.possible_fills.get(index).unwrap().len())
            .unwrap()
            .clone();

        let mut possibilities: Vec<Word> = self.possible_fills[&most_constrained].clone();
        if possibilities.is_empty() {
            return false;
        }

        // possibilities.sort_by_key(|word| self.score_fill(most_constrained, word));
        let mut rng = thread_rng();
        rng.shuffle(&mut possibilities);

        for word in possibilities.iter().take(5) {
            self.fill(most_constrained, word);
            if self.solve() {
                return true;
            }
            self.undo_last_fill();
        }

        false
    }

    pub fn new_grid(&mut self, grid: &Grid) {
        *self = GridSolver::new(grid, &self.dict);
    }
}

impl fmt::Display for GridSolver {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.grid)
    }
}
