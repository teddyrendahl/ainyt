use std::borrow::Cow;
use std::str::from_utf8;

use clap::Parser;
use wordle::web::WordleWebDriver;
use wordle::{Correctness, Guess, Guesser};

#[derive(Parser)]
struct Opts {}

#[tokio::main]
async fn main() {
    let _opts = Opts::parse();
    let driver = WordleWebDriver::create()
        .await
        .expect("Failed to create WebDriver");
    let mut guesser = wordle::WordleSolver::new();
    let mut guess_history = Vec::new();
    for i in 1..=6 {
        let guess = guesser.guess(&guess_history);
        let guess_str = from_utf8(&guess)
            .expect("Guess in not utf8 string!")
            .to_ascii_uppercase();
        let mask = driver
            .guess(&guess_str, i)
            .await
            .expect("Unable to make guess");
        // Print mask result
        println!(
            "Guessed: {}",
            guess_str
                .chars()
                .zip(mask)
                .map(|(c, m)| {
                    match m {
                        Correctness::Correct => format!("\x1b[92;1m{}\x1b[0m", c),
                        Correctness::Misplaced => format!("\x1b[33;1m{}\x1b[0m", c),
                        Correctness::Wrong => format!("\x1b[37;1m{}\x1b[0m", c),
                    }
                })
                .collect::<String>()
        );
        // Win condition
        if mask.iter().all(|c| c == &Correctness::Correct) {
            println!("Puzzle complete, Word was {guess_str}");
            return;
        }
        guess_history.push(Guess {
            word: Cow::Owned(guess),
            mask,
        })
    }
}
