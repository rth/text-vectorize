// Copyright 2019 vtext developers
//
// Licensed under the Apache License, Version 2.0,
// <http://apache.org/licenses/LICENSE-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

/*!
# Vectorization module

This module allows computing a sparse document term matrix from a text corpus.

```rust
extern crate vtext;

use vtext::vectorize::CountVectorizer;
let documents = vec![
    String::from("Some text input"),
    String::from("Another line"),
];

let mut vectorizer = CountVectorizer::new();
let X = vectorizer.fit_transform(&documents);
// returns a sparse CSR matrix with document-terms counts
*/

use crate::math::CSRArray;
use crate::tokenize;
use crate::tokenize::Tokenizer;
use hashbrown::{HashMap, HashSet};
use itertools::sorted;
use ndarray::Array;
use rayon::prelude::*;
use sprs::CsMat;
use std::cmp;

const TOKEN_PATTERN_DEFAULT: &str = r"\b\w\w+\b";

#[cfg(test)]
mod tests;

/// Sort features by name
///
/// Returns a reordered matrix and modifies the vocabulary in place
fn _sort_features(X: &mut CSRArray, vocabulary: &mut HashMap<String, i32>) {
    let mut vocabulary_sorted: Vec<_> = vocabulary
        .iter()
        .map(|(key, val)| (key.clone(), val.clone()))
        .collect();
    vocabulary_sorted.sort_unstable();
    //vocabulary = vocabulary_sorted.into_iter().collect();
    let mut idx_map: Array<usize, _> = Array::zeros(vocabulary_sorted.len());
    for (idx_new, (_term, idx_old)) in vocabulary_sorted.iter().enumerate() {
        idx_map[*idx_old as usize] = idx_new;
        vocabulary
            .entry(_term.to_string())
            .and_modify(|e| *e = idx_new as i32);
    }
    for idx in 0..X.indices.len() {
        X.indices[idx] = idx_map[X.indices[idx]];
    }
}

/// Sum duplicates
#[inline]
fn _sum_duplicates(tf: &mut CSRArray, indices_local: &[i32], nnz: &mut usize) {
    if indices_local.len() > 0 {
        let mut bucket: i32 = 0;
        let mut index_last = indices_local[0];

        for index_current in indices_local.iter().skip(1) {
            bucket += 1;
            if *index_current != index_last {
                tf.indices.push(index_last as usize);
                tf.data.push(bucket);
                *nnz += 1;
                index_last = *index_current;
                bucket = 0;
            }
        }
        tf.indices
            .push(indices_local[indices_local.len() - 1] as usize);
        if bucket == 0 {
            bucket += 1
        }
        tf.data.push(bucket);
    }
    *nnz += 1;

    tf.indptr.push(*nnz);
}

#[derive(Debug)]
pub struct CountVectorizer {
    lowercase: bool,
    token_pattern: String,
    // vocabulary uses i32 indices, to avoid memory copies when converting
    // to sparse CSR arrays in Python with scipy.sparse
    pub vocabulary: HashMap<String, i32>,
}

pub enum Vectorizer {}

impl CountVectorizer {
    /// Initialize a CountVectorizer estimator
    pub fn new() -> Self {
        CountVectorizer {
            lowercase: true,
            token_pattern: String::from(TOKEN_PATTERN_DEFAULT),
            vocabulary: HashMap::with_capacity_and_hasher(1000, Default::default()),
        }
    }

    /// Fit the estimator
    ///
    /// This lists the vocabulary
    pub fn fit(&mut self, X: &[String]) -> () {
        let tokenizer = tokenize::RegexpTokenizer::new(TOKEN_PATTERN_DEFAULT.to_string());

        let tokenize = |X: &[String]| -> HashSet<String> {
            let mut _vocab: HashSet<String> = HashSet::with_capacity(1000);

            for doc in X {
                let doc = doc.to_ascii_lowercase();
                let tokens = tokenizer.tokenize(&doc);

                for token in tokens {
                    if !_vocab.contains(token) {
                        _vocab.insert(token.to_string());
                    };
                }
            }
            _vocab
        };

        let chunk_size = cmp::max(X.len() / 4, 1);

        let pipe = X.par_chunks(chunk_size).flat_map(tokenize);

        let mut vocabulary: HashSet<String> = pipe.collect();

        if vocabulary.len() > 0 {
            self.vocabulary = sorted(vocabulary.iter())
                .zip((0..vocabulary.len()))
                .map(|(tok, idx)| (tok.to_owned(), idx as i32))
                .collect();
        }
    }

    /// Transform
    ///
    /// Converts a sequence of text documents to a CSR Matrix
    pub fn transform(&mut self, X: &[String]) -> CsMat<i32> {
        let mut tf = crate::math::CSRArray {
            indices: Vec::new(),
            indptr: Vec::new(),
            data: Vec::new(),
        };

        tf.indptr.push(0);

        let mut nnz: usize = 0;
        let mut indices_local: Vec<i32> = Vec::new();

        let tokenizer = tokenize::RegexpTokenizer::new(TOKEN_PATTERN_DEFAULT.to_string());

        let pipe = X.iter().map(|doc| doc.to_ascii_lowercase());

        let mut vocabulary_size: i32 = 0;

        for document in pipe {
            let tokens = tokenizer.tokenize(&document);

            indices_local.clear();

            for token in tokens {
                match self.vocabulary.get(token) {
                    Some(_id) => indices_local.push(*_id),
                    None => {}
                };
            }
            // this takes 10-15% of the compute time
            indices_local.sort_unstable();
            _sum_duplicates(&mut tf, indices_local.as_slice(), &mut nnz);
        }

        CsMat::new(
            (tf.indptr.len() - 1, self.vocabulary.len()),
            tf.indptr,
            tf.indices,
            tf.data,
        )
    }

    /// Fit and transform
    ///
    /// This is a single pass vectorization
    pub fn fit_transform(&mut self, X: &[String]) -> CsMat<i32> {
        let mut tf = crate::math::CSRArray {
            indices: Vec::new(),
            indptr: Vec::new(),
            data: Vec::new(),
        };

        tf.indptr.push(0);

        let mut nnz: usize = 0;
        let mut indices_local: Vec<i32> = Vec::new();

        let tokenizer = tokenize::RegexpTokenizer::new(TOKEN_PATTERN_DEFAULT.to_string());

        let pipe = X.iter().map(|doc| doc.to_ascii_lowercase());

        let mut vocabulary_size: i32 = 0;

        for document in pipe {
            let tokens = tokenizer.tokenize(&document);

            indices_local.clear();

            for token in tokens {
                match self.vocabulary.get(token) {
                    Some(_id) => indices_local.push(*_id),
                    None => {
                        self.vocabulary.insert(token.to_string(), vocabulary_size);
                        indices_local.push(vocabulary_size);
                        vocabulary_size += 1;
                    }
                };
            }
            // this takes 10-15% of the compute time
            indices_local.sort_unstable();
            _sum_duplicates(&mut tf, indices_local.as_slice(), &mut nnz);
        }

        _sort_features(&mut tf, &mut self.vocabulary);

        CsMat::new(
            (tf.indptr.len() - 1, self.vocabulary.len()),
            tf.indptr,
            tf.indices,
            tf.data,
        )
    }
}

#[derive(Debug)]
pub struct HashingVectorizer {
    lowercase: bool,
    token_pattern: String,
    n_features: u64,
    _n_jobs: usize,
    thread_pool: Option<rayon::ThreadPool>,
}

impl HashingVectorizer {
    /// Create a new HashingVectorizer estimator
    pub fn new() -> Self {
        HashingVectorizer {
            lowercase: true,
            token_pattern: String::from(TOKEN_PATTERN_DEFAULT),
            n_features: 1048576,
            _n_jobs: 1,
            thread_pool: None,
        }
    }

    /// Set the number of parallel threads to use
    ///
    /// Note: currently any value n_jobs > 1 will use all available cores.
    pub fn n_jobs(mut self, n_jobs: usize) -> Self {
        self._n_jobs = n_jobs;
        if n_jobs == 1 {
            self.thread_pool = None;
        } else if n_jobs > 1 {
            self.thread_pool = Some(
                rayon::ThreadPoolBuilder::new()
                    .num_threads(n_jobs)
                    .build()
                    .unwrap(),
            );
        } else {
            panic!("n_jobs={} must be > 0", n_jobs);
        }
        self
    }

    /// Fit method
    ///
    /// The vectorizer is stateless, this has no effect
    pub fn fit(self, _X: &[String]) -> Self {
        self
    }

    /// Transform method
    pub fn transform(&self, X: &[String]) -> CsMat<i32> {
        let mut tf = crate::math::CSRArray {
            indices: Vec::new(),
            indptr: Vec::new(),
            data: Vec::new(),
        };

        tf.indptr.push(0);

        let mut nnz: usize = 0;

        let tokenizer = tokenize::RegexpTokenizer::new(TOKEN_PATTERN_DEFAULT.to_string());

        let tokenize_hash = |doc: &str| -> Vec<i32> {
            // Closure to tokenize a document and returns hash indices for each token

            let mut indices_local: Vec<i32> = Vec::with_capacity(10);

            for token in tokenizer.tokenize(doc) {
                // set the RNG seeds to get reproducible hashing
                let hash = seahash::hash_seeded(token.as_bytes(), 1, 1000, 200, 89);
                let hash = (hash % self.n_features) as i32;

                indices_local.push(hash);
            }
            // this takes 10-15% of the compute time
            indices_local.sort_unstable();
            indices_local
        };

        let pipe: Box<Iterator<Item = Vec<i32>>>;

        if self._n_jobs == 1 {
            // Sequential (streaming) pipelines
            pipe = Box::new(
                X.iter()
                    // String.to_lowercase() is very slow
                    // https://www.reddit.com/r/rust/comments/6wbru2/performance_issue_can_i_avoid_of_using_the_slow/
                    // https://github.com/rust-lang/rust/issues/26244
                    // Possibly use: https://github.com/JuliaStrings/utf8proc
                    // http://www.unicode.org/faq/casemap_charprop.html
                    .map(|doc| doc.to_ascii_lowercase())
                    .map(|doc| tokenize_hash(&doc)),
            );
        } else if self._n_jobs > 1 {
            // Parallel pipeline. The scaling is reasonably good, however it uses more
            // memory as all the tokens need to be collected into a Vec

            // TODO: explicitly use self.thread_pool, currently the global thread pool is used
            pipe = Box::new(
                X.par_iter()
                    .map(|doc| doc.to_ascii_lowercase())
                    .map(|doc| tokenize_hash(&doc))
                    .collect::<Vec<Vec<i32>>>()
                    .into_iter(),
            );
        } else {
            panic!("n_jobs={} must be > 0", self._n_jobs);
        }

        for indices_local in pipe {
            _sum_duplicates(&mut tf, indices_local.as_slice(), &mut nnz);
        }

        CsMat::new(
            (tf.indptr.len() - 1, self.n_features as usize),
            tf.indptr,
            tf.indices,
            tf.data,
        )
    }

    /// Fit and transform
    ///
    pub fn fit_transform(&self, X: &[String]) -> CsMat<i32> {
        self.transform(X)
    }
}
