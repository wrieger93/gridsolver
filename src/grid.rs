use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fs::File;
use std::io::prelude::*;
use std::io;
use std::iter::Iterator;
use std::path::Path;
use try_from::TryFrom;

use rand::{thread_rng, Rng};

use basic_types::*;
use dict::{UnrankedDict, RankedDict}; 

// Grid
// a grid of cells
// an entry in the grid is a run of at least three consecutive white cells
// in either the across or down direction

#[derive(Clone, Debug)]
pub struct Grid {
    // the cells
    cells: Vec<Cell>,
    // a map of an entryindex to the coordinates of that entry, in order
    entries: HashMap<EntryIndex, Vec<GridCoord>>,
    // all the entries that intersect a given entry
    perpendicular_entries: HashMap<EntryIndex, Vec<EntryIndex>>,
    // duh
    width: usize,
    height: usize,
}

impl Grid {
    // construct a new empty Grid
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

    // construct a Grid from a slice of Cells
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

    // load a Grid from a file
    // see the examples in the assets folder for examples
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
        let height = dimensions[0];
        let width = dimensions[1];

        let cells: Vec<Cell> = entire.chars()
            .skip_while(|c| *c != '\n')
            .skip(1)
            .filter_map(|c| match c {
                '.' => Some(Cell::White(None)),
                '#' => Some(Cell::Black),
                e if e.is_whitespace() => None,
                e => Some(Cell::White(Letter::try_from(e as u8).ok())),
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

    // the coordinates for a given entry
    pub fn get_entry_coords(&self, index: EntryIndex) -> Option<Vec<GridCoord>> {
        self.entries.get(&index).cloned()
    }

    // returns the entry for a given entryindex
    pub fn get_entry(&self, index: EntryIndex) -> Option<Entry> {
        // get the coordinates of the entry
        self.entries.get(&index)
            .map(|coords| {
                // get the cells at those coords
                // and map each cell to the underlying option<letter>
                // all the coords had better point to white cells or we did something wrong
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

    // set an entry to equal a given entry
    pub fn set_entry(&mut self, index: EntryIndex, entry: &Entry) {
        if self.entries.contains_key(&index) {
            let coords: Vec<GridCoord> = self.entries[&index].clone();
            for (coord, letter) in coords.into_iter().zip(entry.letters.iter()) { 
                let new_cell = Cell::White(*letter);
                self.set_cell(coord, new_cell);
            }
        }
    }

    // fill an entry with the given word
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

    // erase all filled cells in the given entry
    pub fn clear_entry(&mut self, index: EntryIndex) {
        if self.entries.contains_key(&index) {
            let coords: Vec<GridCoord> = self.entries[&index].clone();
            for coord in coords { 
                self.set_cell(coord, Cell::White(None));
            }
        }
    }

    // get a list of entries perpendicular to the given one
    pub fn entries_perp_to(&self, index: EntryIndex) -> Vec<EntryIndex> {
        self.perpendicular_entries[&index].clone()
    }

    // check if an entry is filled
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

    // check if the entire grid is filled
    pub fn is_filled(&self) -> bool {
        self.cells.iter().all(|cell| cell.is_black() || cell.is_filled())
    }

    // returns all the entryindex's in the grid in arbitrary order
    pub fn entry_indices(&self) -> Vec<EntryIndex> {
        self.entries.keys().cloned().collect()
    }

    // returns all entry's in the grid in arbitrary order
    pub fn entries(&self) -> Vec<Entry> {
        self.entries.keys()
            .map(|i| self.get_entry(*i).unwrap())
            .collect()
    }

    // returns all the rows of the grid, where a row is a vector of cells
    pub fn rows(&self) -> Vec<Vec<Cell>> {
        (0..self.height)
            .map(|row| {
                (0..self.width)
                    .map(|col| self.get_cell((row, col).into()).unwrap())
                    .collect()
            })
            .collect()
    }

    // returns all the columns of the grid, where a column is a vector of cells
    pub fn cols(&self) -> Vec<Vec<Cell>> {
        (0..self.width)
            .map(|col| {
                (0..self.height)
                    .map(|row| self.get_cell((row, col).into()).unwrap())
                    .collect()
            })
            .collect()
    }

    // converts a coordinate to an index for the self.cells vector
    #[inline]
    fn coord_to_index(&self, coord: GridCoord) -> usize {
        coord.row * self.width + coord.col
    }

    // calculates the entry coordinates for across entries
    // by iterating over the rows
    fn across_entry_coords(&self) -> Vec<Vec<GridCoord>> {
        let mut entry_coords_vec = vec![];
        for (row, row_vec) in self.rows().iter().enumerate() {
            let mut in_entry = false;
            let mut entry_coords = vec![];
            for (col, &cell) in row_vec.iter().enumerate() {
                match cell {
                    Cell::White(_) => {
                        // if we're on a white cell, we're in a possible entry
                        in_entry = true;
                        entry_coords.push((row, col).into());
                    },
                    Cell::Black => {
                        // if we hit a black cell, our entry stops
                        // so check if it's at least length 3 and add it to the list
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
            // need this final check for the end of the row
            if in_entry && entry_coords.len() >= 3 {
                entry_coords_vec.push(entry_coords.clone());
            }
        }
        entry_coords_vec.sort();
        entry_coords_vec
    }

    // calculates the entry coordinates for down entries
    // by iterating over the columns
    fn down_entry_coords(&self) -> Vec<Vec<GridCoord>> {
        // see the comments for across_entry_coords
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

    // rebuilds all the data structures in the grid
    fn rebuild(&mut self) {
        // rebuild the self.entries map
        self.entries.clear();
        let across = self.across_entry_coords();
        let down = self.down_entry_coords();
        let mut entry_counter = 1;
        // iterate over every cell from left to right, top to bottom,
        // and check if it's the start of any entry
        // if it is, add the entry to the map and increment the entry counter
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

        // rebuild the perpendicular entries
        // for every entry, loop over all other entries and see if they have any
        // indices in common
        // there's probably a faster way to do this but it's not a bottleneck
        // since it's only called when the grid is initialized
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
        let words = self.entries().len();
        let average_length = 0;
        write!(f, "{}x{}, {} words, {} average length\n", self.height, self.width, words, average_length)?;
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

// GridSolver
// a structure that fills a grid with valid words from a dictionary

#[derive(Clone, Debug)]
pub struct GridSolver<T: UnrankedDict> {
    // the grid being filled
    grid: Grid,
    // the dictionary in use
    dict: T,
    // words that have been added to the grid already
    // you can't reuse words in a fill
    pub added_words: HashSet<Word>,
    // entries that haven't been filled yet
    unfilled_entries: HashSet<EntryIndex>,
    // for every entry, a list of words that can fill that entry
    // kept up to date as the grid is being filled
    possible_fills: HashMap<EntryIndex, Vec<Word>>,
    // a stack that keeps tract of the changes we make to the grid
    // whenever we insert a new word
    // this allows us to easily backtrack by undoing the changes
    changes: Vec<(EntryIndex, Word, Entry)>,
}

impl<T: UnrankedDict> GridSolver<T> {
    // construct a new gridsolver for the given grid with the given dictionary
    pub fn new(grid: Grid, dict: T) -> GridSolver<T> {
        let mut solver = GridSolver {
            grid: grid,
            dict: dict,
            added_words: HashSet::new(),
            unfilled_entries: HashSet::new(),
            possible_fills: HashMap::new(),
            changes: vec![],
        };

        // all entries are initially unsolved
        for index in solver.grid.entry_indices() {
            if solver.grid.is_entry_filled(index) {
                let entry = solver.grid.get_entry(index).unwrap();
                let letters = entry.letters.into_iter().filter_map(|x| x).collect::<Vec<_>>();
                solver.added_words.insert(Word::new(&letters));
            }
            else {
                solver.update_possible_fills(index);
                solver.unfilled_entries.insert(index);
            }
        }

        solver
    }

    // update the list of possible words for a given index
    fn update_possible_fills(&mut self, index: EntryIndex) {
        // get the entry from the grid
        let opt_entry = self.grid.get_entry(index);
        match opt_entry {
            Some(entry) => {
                // make a pattern fitting the entry
                // and update the possible fill words
                let pattern = Pattern::new(&entry.letters);
                let fills = self.dict.lookup(&pattern);
                self.possible_fills.insert(index, fills);
            }
            _ => {},
        };
    }

    // fill the given entry with the given word
    fn fill(&mut self, index: EntryIndex, word: &Word) {
        // push the index we're changing as well as a copy of the entry before
        // we insert the word onto the changes stack
        self.changes.push((index, word.clone(), self.grid.get_entry(index).unwrap()));
        // fill the entry and remove the index from unfilled_entries
        self.grid.fill_entry(index, word);
        self.unfilled_entries.remove(&index);
        self.added_words.insert(word.clone());
        // update the possible words for the intersecting entries
        for perp in self.grid.entries_perp_to(index) {
            self.update_possible_fills(perp);
        }
    }

    // undo filling the last entry
    fn undo_last_fill(&mut self) {
        // no changes = nothing to undo
        if self.changes.is_empty() {
            return;
        }
        // set the entry to what it was beforehand
        let (index, prev_word, prev_entry) = self.changes.pop().unwrap();
        self.grid.set_entry(index, &prev_entry);
        // the entry is now unfilled
        self.unfilled_entries.insert(index);
        self.added_words.remove(&prev_word);
        // update the possible words for both the index and all intersecting indices
        self.update_possible_fills(index);
        for perp in self.grid.entries_perp_to(index) {
            self.update_possible_fills(perp);
        }
    }

    // fill the grid completely
    // returns true if it's filled, false otherwise
    pub fn solve(&mut self) -> bool {
        // if there are no unfilled entries, we're done
        if self.unfilled_entries.is_empty() {
            return true;
        }
        
        // find entry with the least number of possible fills
        let most_constrained = self.unfilled_entries.iter()
            .min_by_key(|index| self.possible_fills.get(index).unwrap().len())
            .unwrap()
            .clone();

        // if there are zero possible fills, the grid cannot be filled
        let mut possibilities: Vec<Word> = self.possible_fills[&most_constrained].clone();
        if possibilities.is_empty() {
            return false;
        }

        // shuffle the possibile words
        let mut rng = thread_rng();
        rng.shuffle(&mut possibilities);

        // try a different number of possible words based on the length of the words
        // this is completely arbitrary
        let word_len = possibilities[0].size();
        let to_take: usize = if word_len > 8 { 5 } else if word_len > 4 { 5 } else { 5 };

        let possibilities = possibilities.into_iter()
            .take(to_take)
            .collect::<Vec<_>>();

        // for each word to try, insert that word and recursively try filling the grid
        for word in &possibilities {
            // let score = self.dict.get_score(&word).unwrap();
            self.fill(most_constrained, word);
            if self.solve() {
                return true;
            }
            self.undo_last_fill();
        }

        // if none of the words work we can't fill the grid
        false
    }
}

impl<T: RankedDict> GridSolver<T> {
    pub fn average_score(&self) -> f32 {
        let mut score = 0;
        for word in &self.added_words {
            score += self.dict.get_score(&word).unwrap_or(0);
        }
        if !self.added_words.is_empty() {
            (score as f32) / (self.added_words.len() as f32)
        } else {
            0f32
        }
    }

    fn update_possible_fills_ranked(&mut self, index: EntryIndex) {
        // get the entry from the grid
        let opt_entry = self.grid.get_entry(index);
        match opt_entry {
            Some(entry) => {
                // make a pattern fitting the entry
                // and update the possible fill words
                let pattern = Pattern::new(&entry.letters);
                let fills = self.dict.lookup_range(&pattern, Some(40), None);
                self.possible_fills.insert(index, fills);
            }
            _ => {},
        };
    }

    pub fn solve_ranked(&mut self) -> bool {
        // if there are no unfilled entries, we're done
        if self.unfilled_entries.is_empty() {
            return true;
        }
        
        // find entry with the least number of possible fills
        let most_constrained = self.unfilled_entries.iter()
            .min_by_key(|index| self.possible_fills.get(index).unwrap().len())
            .unwrap()
            .clone();

        // if there are zero possible fills, the grid cannot be filled
        let mut possibilities: Vec<Word> = self.possible_fills[&most_constrained].clone();
        if possibilities.is_empty() {
            return false;
        }

        // shuffle the possibile words
        // let mut rng = thread_rng();
        // rng.shuffle(&mut possibilities);

        // try a different number of possible words based on the length of the words
        // this is completely arbitrary
        let word_len = possibilities[0].size();
        let to_take: usize = if word_len > 8 { 5 } else if word_len > 4 { 5 } else { 5 };

        let possibilities = possibilities.into_iter()
            .take(to_take)
            .collect::<Vec<_>>();

        // for each word to try, insert that word and recursively try filling the grid
        for word in &possibilities {
            // let score = self.dict.get_score(&word).unwrap();
            self.fill_ranked(most_constrained, word);
            if self.solve_ranked() {
                return true;
            }
            self.undo_last_fill_ranked();
        }

        // if none of the words work we can't fill the grid
        false
    }

    fn fill_ranked(&mut self, index: EntryIndex, word: &Word) {
        // push the index we're changing as well as a copy of the entry before
        // we insert the word onto the changes stack
        self.changes.push((index, word.clone(), self.grid.get_entry(index).unwrap()));
        // fill the entry and remove the index from unfilled_entries
        self.grid.fill_entry(index, word);
        self.unfilled_entries.remove(&index);
        self.added_words.insert(word.clone());
        // update the possible words for the intersecting entries
        self.update_possible_fills_ranked(index);
        for perp in self.grid.entries_perp_to(index) {
            self.update_possible_fills(perp);
        }
    }

    fn undo_last_fill_ranked(&mut self) {
        // no changes = nothing to undo
        if self.changes.is_empty() {
            return;
        }
        // set the entry to what it was beforehand
        let (index, prev_word, prev_entry) = self.changes.pop().unwrap();
        self.grid.set_entry(index, &prev_entry);
        // the entry is now unfilled
        self.unfilled_entries.insert(index);
        self.added_words.remove(&prev_word);
        // update the possible words for both the index and all intersecting indices
        self.update_possible_fills_ranked(index);
        for perp in self.grid.entries_perp_to(index) {
            self.update_possible_fills_ranked(perp);
        }
    }
}

impl<T: UnrankedDict> fmt::Display for GridSolver<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.grid)?;
        let mut added_words = self.added_words.iter().cloned().collect::<Vec<_>>();
        if added_words.is_empty() {
            return write!(f, "no words added yet\n");
        } else {
            write!(f, "number of words: {}\n", self.added_words.len())?;
        }
        added_words.sort_by_key(|word| word.size());
        let mut prev_word_size = added_words.get(0).unwrap().size();
        for word in &added_words {
            if prev_word_size < word.size() {
                write!(f, "\n")?;
                prev_word_size = word.size();
            }
            write!(f, "{}, ", word)?;
        }
        write!(f, "\n")?;
        Ok(())
    }
}
