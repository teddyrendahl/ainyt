use std::{borrow::Cow, collections::HashSet};
pub mod algorithm;
pub use algorithm::WordleSolver;
pub mod web;

const DICTIONARY: &str = include_str!("../dictionary.txt");

pub type Word = [u8; 5];

pub struct Wordle {
    dictionary: HashSet<&'static Word>,
}

impl Default for Wordle {
    fn default() -> Self {
        Self::new()
    }
}

impl Wordle {
    pub fn new() -> Self {
        Self {
            dictionary: HashSet::from_iter(DICTIONARY.lines().map(|l| {
                l.split_once(' ')
                    .expect("Every line is word + space + frequency")
                    .0
                    .as_bytes()
                    .try_into()
                    .expect("Every word must be 5 bytes")
            })),
        }
    }

    // Play six rounds where it invokes the Guesser each round
    pub fn play<G: Guesser>(&self, answer: &'static Word, mut guesser: G) -> Option<usize> {
        let mut history = Vec::new();
        // Wordle only allows 6 guesses but we want to allow more so we can see the peformance tail in failure cases
        for i in 1..=32 {
            let guess: [u8; 5] = guesser.guess(&history);
            if &guess == answer {
                return Some(i);
            }
            assert!(self.dictionary.contains(&guess));
            let correctness = Correctness::compute(answer, &guess);
            history.push(Guess {
                word: Cow::Owned(guess),
                mask: correctness,
            })
        }
        None
    }
}

impl Correctness {
    pub fn compute(answer: &Word, guess: &Word) -> [Self; 5] {
        assert_eq!(answer.len(), 5);
        assert_eq!(guess.len(), 5);
        let mut c = [Correctness::Wrong; 5];
        let mut used = [false; 5];

        for (i, (a, g)) in answer.iter().zip(guess.iter()).enumerate() {
            if a == g {
                c[i] = Correctness::Correct;
                used[i] = true;
            }
        }
        for (i, g) in guess.iter().enumerate() {
            if c[i] == Correctness::Correct {
                continue;
            }
            if answer.iter().enumerate().any(|(i, a)| {
                if g == a && !used[i] {
                    used[i] = true;
                    true
                } else {
                    false
                }
            }) {
                c[i] = Correctness::Misplaced
            }
        }
        c
    }

    pub fn permutations() -> impl Iterator<Item = [Self; 5]> {
        itertools::iproduct!(
            [Self::Correct, Self::Misplaced, Self::Wrong],
            [Self::Correct, Self::Misplaced, Self::Wrong],
            [Self::Correct, Self::Misplaced, Self::Wrong],
            [Self::Correct, Self::Misplaced, Self::Wrong],
            [Self::Correct, Self::Misplaced, Self::Wrong]
        )
        .map(|(a, b, c, d, e)| [a, b, c, d, e])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Correctness {
    Correct,   // green
    Misplaced, // yellow
    Wrong,
}

pub struct Guess<'a> {
    pub word: Cow<'a, Word>,
    pub mask: [Correctness; 5],
}

impl Guess<'_> {
    pub fn matches(&self, word: &Word) -> bool {
        Correctness::compute(word, &self.word) == self.mask
    }
}
pub trait Guesser {
    fn guess(&mut self, history: &[Guess]) -> Word;
}

#[cfg(test)]
macro_rules! mask {
    (C) => {crate::Correctness::Correct};
    (M) => {crate::Correctness::Misplaced};
    (W) => {crate::Correctness::Wrong};
    ($($c:tt)+) => {[$(mask!($c)),+]}
}

#[cfg(test)]
mod tests {
    mod guess_matches {
        use crate::Guess;
        use std::borrow::Cow;

        macro_rules! check {
            ($prev:literal + [$($mask:tt)+] allows $next:literal) => {
                assert!(Guess {
                    word: Cow::Borrowed($prev),
                    mask: mask![$($mask)+]
                }.matches($next))
            };
            ($prev:literal + [$($mask:tt)+] disallows $next:literal) => {
                assert!(!Guess {
                    word: Cow::Borrowed($prev),
                    mask: mask![$($mask)+]
                }.matches($next))
            };
        }

        #[test]
        fn matches() {
            check!(b"abcde" + [C C C C C] allows b"abcde");
            check!(b"abcdf" + [C C C C C] disallows b"abcde");
            check!(b"abcde" + [W W W W W] allows b"fghij");
            check!(b"abcde" + [M M M M M] allows b"eabcd");
            check!(b"aaabb" + [C M W W W] disallows b"accaa");
            check!(b"baaaa" + [W C M W W] allows b"aaccc");
            check!(b"baaaa" + [W C M W W] disallows b"caacc");
            check!(b"abcde" + [W W W W W] disallows b"bcdea");
            check!(b"tares" + [W M M W W] disallows b"brink");
        }
    }
    mod game {
        use crate::{Guess, Wordle};

        macro_rules! guesser {
            (|$history:ident| $impl:block) => {{
                struct G;
                impl $crate::Guesser for G {
                    fn guess(&mut self, $history: &[Guess]) -> $crate::Word {
                        $impl
                    }
                }
                G
            }};
        }
        #[test]
        fn genius() {
            let wordle = Wordle::new();
            let guesser = guesser!(|_history| { *b"moved" });
            assert_eq!(wordle.play(b"moved", guesser), Some(1));
        }

        #[test]
        fn magnificent() {
            let wordle = Wordle::new();
            let guesser = guesser!(|history| {
                if history.len() == 1 {
                    *b"right"
                } else {
                    *b"wrong"
                }
            });
            assert_eq!(wordle.play(b"right", guesser), Some(2));
        }
        #[test]
        fn impressive() {
            let wordle = Wordle::new();
            let guesser = guesser!(|history| {
                if history.len() == 2 {
                    *b"right"
                } else {
                    *b"wrong"
                }
            });
            assert_eq!(wordle.play(b"right", guesser), Some(3));
        }
        #[test]
        fn splendid() {
            let wordle = Wordle::new();
            let guesser = guesser!(|history| {
                if history.len() == 3 {
                    *b"right"
                } else {
                    *b"wrong"
                }
            });
            assert_eq!(wordle.play(b"right", guesser), Some(4));
        }

        #[test]
        fn great() {
            let wordle = Wordle::new();
            let guesser = guesser!(|history| {
                if history.len() == 4 {
                    *b"right"
                } else {
                    *b"wrong"
                }
            });
            assert_eq!(wordle.play(b"right", guesser), Some(5));
        }

        #[test]
        fn phew() {
            let wordle = Wordle::new();
            let guesser = guesser!(|history| {
                if history.len() == 5 {
                    *b"right"
                } else {
                    *b"wrong"
                }
            });
            assert_eq!(wordle.play(b"right", guesser), Some(6));
        }

        #[test]
        fn oops() {
            let wordle = Wordle::new();
            let guesser = guesser!(|_history| { *b"wrong" });
            assert_eq!(wordle.play(b"right", guesser), None);
        }
    }
    mod correctness {
        use crate::Correctness;

        #[test]
        fn all_green() {
            assert_eq!(Correctness::compute(b"abcde", b"abcde"), mask![C C C C C]);
        }

        #[test]
        fn all_grey() {
            assert_eq!(Correctness::compute(b"abcde", b"fghij"), mask![W W W W W]);
        }

        #[test]
        fn all_yellow() {
            assert_eq!(Correctness::compute(b"abcde", b"eabcd"), mask![M M M M M]);
        }

        #[test]
        fn repeat_green() {
            assert_eq!(Correctness::compute(b"aabbb", b"aaccc"), mask![C C W W W]);
        }

        #[test]
        fn repeat_yellow() {
            assert_eq!(Correctness::compute(b"aabbb", b"ccaac"), mask![W W M M W]);
        }

        #[test]
        fn repeat_some_green() {
            assert_eq!(Correctness::compute(b"aabbb", b"caacc"), mask![W C M W W]);
        }

        #[test]
        fn only_some_yellows() {
            assert_eq!(Correctness::compute(b"azzaz", b"aaabb"), mask![C M W W W]);
        }

        #[test]
        fn misplaced_before_correct_not_yellow() {
            assert_eq!(Correctness::compute(b"baccc", b"aaddd"), mask![W C W W W]);
        }

        #[test]
        fn one_green() {
            assert_eq!(Correctness::compute(b"abcde", b"aacde"), mask![C W C C C]);
        }
    }
}
