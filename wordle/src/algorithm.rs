use std::borrow::Cow;

use crate::{Correctness, Guess, Guesser, Word};

pub struct WordleSolver {
    remaining: Vec<(&'static Word, usize)>,
    patterns: Vec<[Correctness; 5]>,
}

impl Default for WordleSolver {
    fn default() -> Self {
        Self::new()
    }
}
impl WordleSolver {
    pub fn new() -> Self {
        Self {
            remaining: Vec::from_iter(crate::DICTIONARY.lines().map(|l| {
                let (word, count) = l
                    .split_once(' ')
                    .expect("Every line is word + space + frequency");
                let count: usize = count.parse().expect("every count is a number");
                let word = word
                    .as_bytes()
                    .try_into()
                    .expect("every dictionary  word is 5 characters");
                (word, count)
            })),
            patterns: Correctness::permutations().collect(),
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct Candidate {
    word: &'static Word,
    goodness: f64,
}

impl Guesser for WordleSolver {
    fn guess(&mut self, history: &[Guess]) -> Word {
        if history.is_empty() {
            return *b"crate";
        }
        if let Some(last) = history.last() {
            self.remaining.retain(|(word, _count)| last.matches(word));
        }
        let remaining_count: usize = self.remaining.iter().map(|&(_, c)| c).sum();

        let mut best: Option<Candidate> = None;
        for &(word, count) in &self.remaining {
            // consider a world where we did guess word and got pattern
            // as the Correctness match. Now, compute what then is left.
            let mut sum = 0.0;
            self.patterns.retain(|pattern| {
                let mut in_pattern_total = 0;
                for (candidate, c) in &self.remaining {
                    let g = Guess {
                        word: Cow::Borrowed(word),
                        mask: *pattern,
                    };
                    if g.matches(candidate) {
                        in_pattern_total += c;
                    }
                }
                if in_pattern_total == 0 {
                    return false;
                }
                // TODO: apply sigmoid
                let p_of_pattern = in_pattern_total as f64 / remaining_count as f64;
                sum += p_of_pattern * p_of_pattern.log2();
                true
            });
            // This weights the "goodness" by the probability this is the answer.
            // This can be removed and we will purely favor words that provide
            // us more information
            let p_word = count as f64 / remaining_count as f64;
            let goodness = -sum * p_word;
            if let Some(c) = best {
                // Is this one better
                if goodness > c.goodness {
                    best = Some(Candidate { word, goodness })
                }
            } else {
                best = Some(Candidate { word, goodness })
            }
        }
        *best.unwrap().word
    }
}
