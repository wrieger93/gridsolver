use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::prelude::*;
use std::io;
use std::path::Path;

use basic_types::*;

pub trait UnrankedDict: Sized {
    fn from_file<P: AsRef<Path>>(path: P) -> io::Result<Self>;
    fn add(&mut self, word: &Word);
    fn remove(&mut self, word: &Word);
    fn contains(&self, word: &Word) -> bool;
    fn lookup(&self, pattern: &Pattern) -> Vec<Word>;
}

pub trait RankedDict : UnrankedDict {
    fn from_file<P: AsRef<Path>>(path: P) -> io::Result<Self>;
    fn get_score(&self, word: &Word) -> Option<i32>;
    fn set_score(&mut self, word: &Word, rank: i32) -> bool;
    fn lookup_range(&self, pattern: &Pattern, lower: Option<i32>, upper: Option<i32>) -> Vec<Word>;
    fn max_rank(&self) -> i32;
    fn min_rank(&self) -> i32;
}

// Dictionary
// a structure that supports finding words that match a pattern

#[derive(Clone, Debug, Default)]
pub struct Dictionary {
    // a map of word length to all words of that length
    words_by_size: HashMap<usize, HashSet<Word>>,
}

impl Dictionary {
    pub fn new() -> Dictionary {
        Dictionary::default()
    }
}

impl UnrankedDict for Dictionary {
    fn from_file<P: AsRef<Path>>(path: P) -> io::Result<Dictionary> {
        // read the file
        let mut entire = String::new();
        File::open(path)?.read_to_string(&mut entire)?;

        // split the file into words and add them to the dict
        let mut dict = Dictionary::new();
        for word in entire.split('\n').map(Word::from) {
            dict.add(&word);
        }
        Ok(dict)
    }

    // add a word to the dictionary
    fn add(&mut self, word: &Word) {
        self.words_by_size.entry(word.size())
            .or_insert(HashSet::new())
            .insert(word.clone());
    }

    // remove a word from the dictionary
    fn remove(&mut self, word: &Word) {
        self.words_by_size.entry(word.size())
            .or_insert(HashSet::new())
            .remove(word);
    }

    // check if the dictionary contains a word
    fn contains(&self, word: &Word) -> bool {
        match self.words_by_size.get(&word.size()) {
            Some(set) => set.contains(word),
            None => false
        }
    }

    // find all words in the dictionary that match the Pattern
    fn lookup(&self, pattern: &Pattern) -> Vec<Word> {
        // a blank pattern matches every word of that length
        let empty = !pattern.masks.iter().any(|opt| opt.is_some());
        if empty {
            self.words_by_size[&pattern.size()].iter().cloned().collect()
        }

        // actually do the lookup
        else {
            self.words_by_size[&pattern.size()].iter()
                .filter(|w| pattern.matches(w))
                .cloned()
                .collect()
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct RankedDictionary {
    words_by_size: HashMap<usize, HashMap<Word, i32>>,
    default_score: i32,
}

impl RankedDictionary {
    pub fn new() -> RankedDictionary {
        RankedDictionary::default()
    }
}

impl UnrankedDict for RankedDictionary {
    fn from_file<P: AsRef<Path>>(path: P) -> io::Result<RankedDictionary> {
        // read the file
        let mut entire = String::new();
        File::open(path)?.read_to_string(&mut entire)?;

        // split the file into words and add them to the dict
        let mut dict = Self::new();
        for word in entire.split('\n').map(Word::from) {
            dict.add(&word);
        }
        Ok(dict)
    }

    fn add(&mut self, word: &Word) {
        self.words_by_size.entry(word.size())
            .or_insert(HashMap::new())
            .insert(word.clone(), self.default_score);
    }

    fn remove(&mut self, word: &Word) {
        self.words_by_size.entry(word.size())
            .or_insert(HashMap::new())
            .remove(word);
    }

    fn contains(&self, word: &Word) -> bool {
        match self.words_by_size.get(&word.size()) {
            Some(map) => map.contains_key(word),
            None => false,
        }
    }

    fn lookup(&self, pattern: &Pattern) -> Vec<Word> {
        let mut pairs = self.words_by_size.get(&pattern.size()).unwrap().into_iter()
            .filter(|&(w, _)| pattern.matches(w))
            .map(|(w, r)| (w.clone(), r.clone()))
            .collect::<Vec<(Word, i32)>>();
        pairs.sort_by_key(|&(_, rank)| -rank);
        pairs.into_iter().map(|pair| pair.0).collect()
    }
}

impl RankedDict for RankedDictionary {
    fn from_file<P: AsRef<Path>>(path: P) -> io::Result<RankedDictionary> {
        // read the file
        let mut entire = String::new();
        File::open(path)?.read_to_string(&mut entire)?;

        // split the file into words and add them to the dict
        let mut dict = RankedDictionary::new();
        for line in entire.split('\n') {
            let parts = line.splitn(2, ';').collect::<Vec<_>>();
            // println!("{:?}", parts);
            if parts.len() != 2 {
                continue;
            }
            let score = parts[1].trim().parse::<i32>().unwrap();
            let word = Word::from(parts[0]);
            dict.add(&word);
            dict.set_score(&word, score);
        }
        Ok(dict)
    }

    fn get_score(&self, word: &Word) -> Option<i32> {
        match self.words_by_size.get(&word.size()) {
            Some(map) => map.get(word).cloned(),
            None => None,
        }
    }

    fn set_score(&mut self, word: &Word, rank: i32) -> bool {
        if !self.contains(word) {
            return false;
        }
        self.words_by_size.entry(word.size())
            .or_insert(HashMap::new())
            .insert(word.clone(), rank);
        // if let Some(mut map) = self.words_by_size.get_mut(&word.size()) {
        //     map.insert(word.clone(), rank);
        // }
        true
    }

    fn lookup_range(&self, pattern: &Pattern, lower: Option<i32>, upper: Option<i32>) -> Vec<Word> {
        self.words_by_size.get(&pattern.size()).unwrap().iter()
            .filter(|&(word, _)| pattern.matches(word))
            .filter(|&(_, &rank)| {
                if let Some(bound) = lower {
                    bound <= rank 
                } else {
                    true
                }
            })
            .filter(|&(_, &rank)| {
                if let Some(bound) = upper {
                    rank <= bound
                } else {
                    true
                }
            })
            .map(|(w, _)| w)
            .cloned()
            .collect()
    }

    fn max_rank(&self) -> i32 {
        unimplemented!()
    }

    fn min_rank(&self) -> i32 {
        unimplemented!()
    }
}

#[cfg(test)]
mod test {
}
