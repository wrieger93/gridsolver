use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::prelude::*;
use std::io;
use std::path::Path;

use basic_types::*;

// Dictionary

#[derive(Clone, Debug, Default)]
pub struct Dictionary {
    all_words: HashSet<Word>,
    words_by_size: HashMap<usize, Vec<Word>>,
    lookups: HashMap<Pattern, Vec<Word>>,
}

impl Dictionary {
    pub fn new() -> Dictionary {
        Dictionary::default()
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> io::Result<Dictionary> {
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

    pub fn add(&mut self, word: &Word) {
        if !self.all_words.contains(word) {
            self.all_words.insert(word.clone());
            self.words_by_size.entry(word.size())
                .or_insert(vec![])
                .push(word.clone());
        }
    }

    pub fn remove(&mut self, word: &Word) {
        if self.all_words.contains(word) {
            self.all_words.remove(word);
            let vec = self.words_by_size.entry(word.size())
                .or_insert(vec![]);
            let pos = vec.iter().position(|w| w == word);
            if pos.is_some() { // this should always be true
                (*vec).remove(pos.unwrap());
            }
        }
    }

    pub fn contains(&self, word: &Word) -> bool {
        self.all_words.contains(word)
    }

    pub fn lookup(&mut self, pattern: &Pattern) -> Vec<Word> {
        let empty = !pattern.masks.iter().any(|opt| opt.is_some());
        if empty {
            return self.words_by_size[&pattern.size()].clone();
        }

        if self.lookups.contains_key(pattern) {
            self.lookups[pattern].clone()
        } else {
            let res: Vec<Word> = self.words_by_size[&pattern.size()].iter()
                .filter(|w| pattern.matches(w))
                .cloned()
                .collect();
            self.lookups.insert(pattern.clone(), res.clone());
            res
        }
    }
}
