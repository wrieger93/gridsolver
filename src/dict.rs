use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::prelude::*;
use std::io;
use std::path::Path;

use basic_types::*;

// Dictionary
// a structure that supports finding words that match a pattern

#[derive(Clone, Debug, Default)]
pub struct Dictionary {
    // all the words in the dict
    all_words: HashSet<Word>,
    // a map of word length to all words of that length
    words_by_size: HashMap<usize, Vec<Word>>,
    // a cache of all the lookups that have been done
    lookups: HashMap<Pattern, Vec<Word>>,
}

impl Dictionary {
    pub fn new() -> Dictionary {
        Dictionary::default()
    }

    // load a dictionary from a file that's a list of words
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

    // add a word to the dictionary
    pub fn add(&mut self, word: &Word) {
        if !self.all_words.contains(word) {
            self.all_words.insert(word.clone());
            self.words_by_size.entry(word.size())
                .or_insert(vec![])
                .push(word.clone());
        }
    }

    // remove a word from the dictionary
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

    // check if the dictionary contains a word
    pub fn contains(&self, word: &Word) -> bool {
        self.all_words.contains(word)
    }

    // find all words in the dictionary that match the Pattern
    pub fn lookup(&mut self, pattern: &Pattern) -> Vec<Word> {
        // a blank pattern matches every word of that length
        let empty = !pattern.masks.iter().any(|opt| opt.is_some());
        if empty {
            self.words_by_size[&pattern.size()].clone()
        }
        // check if the pattern is in the cache
        else if self.lookups.contains_key(pattern) {
            self.lookups[pattern].clone()
        }
        // actually do the lookup
        else {
            let res: Vec<Word> = self.words_by_size[&pattern.size()].iter()
                .filter(|w| pattern.matches(w))
                .cloned()
                .collect();
            // add this lookup to the cache
            self.lookups.insert(pattern.clone(), res.clone());
            res
        }
    }
}
