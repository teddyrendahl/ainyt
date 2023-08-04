use std::{borrow::Cow, collections::HashMap};

use crate::{Correctness, Guess, Guesser};

pub struct Naive {
    remaining: HashMap<&'static str, usize>,
}

impl Default for Naive {
    fn default() -> Self {
        Self::new()
    }
}
impl Naive {
    pub fn new() -> Self {
        Self {
            remaining: HashMap::from_iter(crate::DICTIONARY.lines().map(|l| {
                let (word, count) = l
                    .split_once(' ')
                    .expect("Every line is word + space + frequency");
                let count: usize = count.parse().expect("every count is a number");
                (word, count)
            })),
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct Candidate {
    word: &'static str,
    goodness: f64,
}

impl Guesser for Naive {
    fn guess(&mut self, history: &[Guess]) -> String {
        if history.is_empty() {
            return "tares".into();
        }
        if let Some(last) = history.last() {
            self.remaining.retain(|word, _count| last.matches(word));
        }
        let remaining_count: usize = self.remaining.iter().map(|(_, &c)| c).sum();

        let mut best: Option<Candidate> = None;
        for (&word, &count) in &self.remaining {
            // consider a world where we did guess word and got pattern
            // as the Correctness match. Now, compute what then is left.
            let mut sum = 0.0;
            for pattern in Correctness::permutations() {
                let mut in_pattern_total = 0;
                for candidate in self.remaining.keys() {
                    let g = Guess {
                        word: Cow::Owned(word.to_string()),
                        mask: pattern,
                    };
                    if g.matches(candidate) {
                        in_pattern_total += count;
                    }
                }
                if in_pattern_total == 0 {
                    continue;
                }
                let p_of_pattern = in_pattern_total as f64 / remaining_count as f64;
                sum += p_of_pattern * p_of_pattern.log2();
            }
            let goodness = -sum;
            if let Some(c) = best {
                // Is this one better
                if goodness > c.goodness {
                    best = Some(Candidate { word, goodness })
                }
            } else {
                best = Some(Candidate { word, goodness })
            }
        }
        best.unwrap().word.into()
    }
}
