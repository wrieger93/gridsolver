use std::fmt;
use try_from::TryFrom;

use unidecode::unidecode;

// Letter
// really just a byte
// A through Z, must be capital

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Letter(u8);

impl TryFrom<u8> for Letter {
    type Err = ();

    // attempt converting a byte to a letter
    // uses unidecode so that e.g. b'á' maps to A
    fn try_from(byte: u8) -> Result<Letter, Self::Err> {
        let as_char = byte as char;
        if byte < 128 && as_char.is_alphabetic() {
            let uppercase: Vec<char> = as_char.to_uppercase().collect();
            Ok(Letter(uppercase[0] as u8))
        } else {
            Err(())
        }
    }
}

impl From<Letter> for u8 {
    // convert a letter to a byte
    fn from(letter: Letter) -> u8 {
        letter.0
    }
}

impl fmt::Display for Letter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0 as char)
    }
}

// Word
// just a vector of letters

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Word {
    pub letters: Vec<Letter>,
}

impl Word {
    // constructs a new word from the slice
    pub fn new(letters: &[Letter]) -> Word {
        Word {
            letters: letters.iter().cloned().collect(),
        }
    }

    // the length of the word
    pub fn size(&self) -> usize {
        self.letters.len()
    }
}

impl<'a> From<&'a str> for Word {
    // converts from a string to a word
    // ignores all non-alphabetic characters
    // e.g. "?;ab.C'D" becomes "ABCD"
    fn from(string: &'a str) -> Word {
        let letters = unidecode(string).bytes()
            .filter_map(|b| Letter::try_from(b).ok())
            .collect();
        Word {
            letters: letters,
        }
    }
}

impl<'a> From<&'a Word> for String {
    fn from(word: &'a Word) -> String {
        word.letters.iter()
            .cloned()
            .map(<u8>::from)
            .map(<char>::from)
            .collect()
    }
}

impl fmt::Display for Word {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", String::from(self))
    }
}

// Pattern
// just a vector of option<letter>
// the pattern "..A." matches "STAN", for example
// . is represented by none
// A is represented by some(Letter(b'A'))

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Pattern {
    pub masks: Vec<Option<Letter>>,
}

impl Pattern {
    // constructs a pattern from the given slice
    pub fn new(masks: &[Option<Letter>]) -> Pattern {
        Pattern {
            masks: masks.iter().cloned().collect(),
        }
    }

    // the length of the pattern
    pub fn size(&self) -> usize {
        self.masks.len()
    }

    // check if a word matches the pattern
    pub fn matches(&self, word: &Word) -> bool {
        // can't match if they're not the same size
        if word.size() != self.size() {
            false
        // make sure every some(letter) matches the corresponding letter in the word
        } else {
            self.masks.iter()
                .zip(word.letters.iter())
                .filter_map(|(mask, letter)| mask.map(|l| l == *letter))
                .all(|x| x)
        }
    }
}

impl<'a> From<&'a str> for Pattern {
    // converts from a string to a pattern
    // ignores all characters other than alphabetic ones and . (period)
    // which represents an empty pattern
    // e.g. "?.A.'" becomes the pattern ".A."
    fn from(string: &'a str) -> Pattern {
        let masks: Vec<Option<Letter>> = unidecode(string).bytes()
            .filter_map(|b| {
                if b == b'.' {
                    Some(None)
                } else {
                    Letter::try_from(b).ok().map(Some)
                }
            })
            .collect();
        Pattern {
            masks: masks,
        }
    }
}

impl fmt::Display for Pattern {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for opt in &self.masks {
            match *opt {
                Some(l) => write!(f, "{}", l),
                None => write!(f, "."),
            }?;
        }
        Ok(())
    }
}

// GridCoord
// a coordinate in a grid
// just a pair of usize

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct GridCoord {
    pub row: usize,
    pub col: usize,
}

impl GridCoord {
    // constructs a new gridcoord
    pub fn new(row: usize, col: usize) -> GridCoord {
        GridCoord {
            row: row,
            col: col,
        }
    }

    // shifts the gridcoord by the given offsets
    pub fn offset(&self, row_offset: i32, col_offset: i32) -> Option<GridCoord> {
        let new_row = (self.row as i32) + row_offset;
        let new_col = (self.col as i32) + col_offset;
        if new_row >= 0 && new_col >= 0 {
            Some(GridCoord::new(new_row as usize, new_col as usize))
        } else {
            None
        }
    }

    // returns the cells neighboring the gridcoord
    pub fn neighbors(&self) -> Vec<GridCoord> {
        [self.offset(0, -1), self.offset(1, 0), self.offset(0, -1), self.offset(-1, 0)]
            .into_iter()
            .filter_map(|opt| *opt)
            .collect()
    }
}

impl From<(usize, usize)> for GridCoord {
    fn from(tuple: (usize, usize)) -> GridCoord {
        GridCoord::new(tuple.0, tuple.1)
    }
}

impl fmt::Display for GridCoord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.row, self.col)
    }
}

// EntryDir
// self explanatory i hope

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EntryDir {
    Across,
    Down,
}

// EntryIndex
// the index of an entry in a grid
// STARTS AT 1, NOT 0
// i did this to conform with crossword numberings
// since they start at 1 across
// but that means i have to use try_from instead of just from :(

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EntryIndex {
    // every entry has a number and a direction
    pub num: u32,
    pub dir: EntryDir,
}

impl EntryIndex {
}

impl TryFrom<(u32, EntryDir)> for EntryIndex {
    type Err = ();

    fn try_from(tuple: (u32, EntryDir)) -> Result<EntryIndex, Self::Err> {
        if tuple.0 > 0 {
            Ok(EntryIndex {
                num: tuple.0,
                dir: tuple.1,
            })
        } else {
            Err(())
        }
    }
}

impl Default for EntryIndex {
    fn default() -> EntryIndex {
        EntryIndex {
            num: 1,
            dir: EntryDir::Across,
        }
    }
}

impl fmt::Display for EntryIndex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let dir = match self.dir {
            EntryDir::Across => "across",
            EntryDir::Down => "down",
        };
        write!(f, "{} {}", self.num, dir)
    }
}

// Entry
// an entry in a grid
// just a vector of option<letter>
// none means that cell is empty, some(letter) is filled with that letter

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Entry {
    pub letters: Vec<Option<Letter>>,
}

impl Entry {
    pub fn new(letters: Vec<Option<Letter>>) -> Entry {
        Entry {
            letters: letters,
        }
    }
}

// Cell
// a cell in a grid
// black cells cannot be filled
// white cells can be filled or unfilled

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Cell {
    Black,
    White(Option<Letter>),
}

impl Cell {
    pub fn is_white(&self) -> bool {
        match *self {
            Cell::White(_) => true,
            _ => false,
        }
    }

    pub fn is_black(&self) -> bool {
        !self.is_white()
    }

    pub fn is_filled(&self) -> bool {
        match *self {
            Cell::White(Some(_)) => true,
            _ => false,
        }
    }
}
