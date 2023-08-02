use std::collections::HashSet;

pub mod algorithms;

const DICTIONARY: &str = include_str!("../dictionary.txt");

pub struct Wordle {
    dictionary: HashSet<&'static str>,
}

impl Wordle {
    pub fn new() -> Self {
        Self {
            dictionary: HashSet::from_iter(DICTIONARY.lines().map(|l| {
                l.split_once(" ")
                    .expect("Every line is word + space + frequency")
                    .0
            })),
        }
    }

    // Play six rounds where it invokes the Guesser each round
    pub fn play<G: Guesser>(&self, answer: &'static str, guesser: &G) -> Option<usize> {
        let mut history = Vec::new();
        // Wordle only allows 6 guesses but we want to allow more so we can see the peformance tail in failure cases
        for i in 1..=32 {
            let guess = guesser.guess(&history);
            if guess == answer {
                return Some(i);
            }
            assert!(self.dictionary.contains(&*guess));
            let correctness = Correctness::compute(answer, &guess);
            history.push(Guess {
                word: guess,
                mask: correctness,
            })
        }
        None
    }
}

impl Correctness {
    pub fn compute(answer: &str, guess: &str) -> [Self; 5] {
        assert_eq!(answer.len(), 5);
        assert_eq!(guess.len(), 5);
        let mut c = [Correctness::Wrong; 5];
        let mut used = [false; 5];

        for (i, (a, g)) in answer.chars().zip(guess.chars()).enumerate() {
            if a == g {
                c[i] = Correctness::Correct;
                used[i] = true;
            }
        }
        for (i, g) in guess.chars().enumerate() {
            if c[i] == Correctness::Correct {
                continue;
            }
            if answer.chars().into_iter().enumerate().any(|(i, a)| {
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Correctness {
    Correct,   // green
    Misplaced, // yellow
    Wrong,
}
pub struct Guess {
    pub word: String,
    pub mask: [Correctness; 5],
}
pub trait Guesser {
    fn guess(&self, history: &[Guess]) -> String;
}

impl Guesser for fn(history: &[Guess]) -> String {
    fn guess(&self, history: &[Guess]) -> String {
        (*self)(history)
    }
}

#[cfg(test)]
mod tests {
    mod game {
        use crate::{Guess, Wordle};

        macro_rules! guesser {
            (|$history:ident| $impl:block) => {{
                struct G;
                impl $crate::Guesser for G {
                    fn guess(&self, $history: &[Guess]) -> String {
                        $impl
                    }
                }
                G
            }};
        }
        #[test]
        fn genius() {
            let wordle = Wordle::new();
            let guesser = guesser!(|_history| { "moved".to_string() });
            assert_eq!(wordle.play("moved", &guesser), Some(1));
        }

        #[test]
        fn magnificent() {
            let wordle = Wordle::new();
            let guesser = guesser!(|history| {
                if history.len() == 1 {
                    "right".into()
                } else {
                    "wrong".into()
                }
            });
            assert_eq!(wordle.play("right", &guesser), Some(2));
        }
        #[test]
        fn impressive() {
            let wordle = Wordle::new();
            let guesser = guesser!(|history| {
                if history.len() == 2 {
                    "right".into()
                } else {
                    "wrong".into()
                }
            });
            assert_eq!(wordle.play("right", &guesser), Some(3));
        }
        #[test]
        fn splendid() {
            let wordle = Wordle::new();
            let guesser = guesser!(|history| {
                if history.len() == 3 {
                    "right".into()
                } else {
                    "wrong".into()
                }
            });
            assert_eq!(wordle.play("right", &guesser), Some(4));
        }

        #[test]
        fn great() {
            let wordle = Wordle::new();
            let guesser = guesser!(|history| {
                if history.len() == 4 {
                    "right".into()
                } else {
                    "wrong".into()
                }
            });
            assert_eq!(wordle.play("right", &guesser), Some(5));
        }

        #[test]
        fn phew() {
            let wordle = Wordle::new();
            let guesser = guesser!(|history| {
                if history.len() == 5 {
                    "right".into()
                } else {
                    "wrong".into()
                }
            });
            assert_eq!(wordle.play("right", &guesser), Some(6));
        }

        #[test]
        fn oops() {
            let wordle = Wordle::new();
            let guesser = guesser!(|history| { "wrong".into() });
            assert_eq!(wordle.play("right", &guesser), None);
        }
    }
    mod correctness {
        use crate::Correctness;

        macro_rules! mask {
            (C) => {Correctness::Correct};
            (M) => {Correctness::Misplaced};
            (W) => {Correctness::Wrong};
            ($($c:tt)+) => {[$(mask!($c)),+]}
        }

        #[test]
        fn all_green() {
            assert_eq!(Correctness::compute("abcde", "abcde"), mask![C C C C C]);
        }

        #[test]
        fn all_grey() {
            assert_eq!(Correctness::compute("abcde", "fghij"), mask![W W W W W]);
        }

        #[test]
        fn all_yellow() {
            assert_eq!(Correctness::compute("abcde", "eabcd"), mask![M M M M M]);
        }

        #[test]
        fn repeat_green() {
            assert_eq!(Correctness::compute("aabbb", "aaccc"), mask![C C W W W]);
        }

        #[test]
        fn repeat_yellow() {
            assert_eq!(Correctness::compute("aabbb", "ccaac"), mask![W W M M W]);
        }

        #[test]
        fn repeat_some_green() {
            assert_eq!(Correctness::compute("aabbb", "caacc"), mask![W C M W W]);
        }

        #[test]
        fn only_some_yellows() {
            assert_eq!(Correctness::compute("azzaz", "aaabb"), mask![C M W W W]);
        }

        #[test]
        fn misplaced_before_correct_not_yellow() {
            assert_eq!(Correctness::compute("baccc", "aaddd"), mask![W C W W W]);
        }

        #[test]
        fn one_green() {
            assert_eq!(Correctness::compute("abcde", "aacde"), mask![C W C C C]);
        }
    }
}
