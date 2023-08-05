use clap::Parser;
use wordle::Wordle;

const GAMES: &str = include_str!("../answers.txt");

#[derive(Parser)]
struct Opt {
    #[clap(short, long)]
    max: Option<usize>,
}

fn main() {
    let args = Opt::parse();
    let wordle = Wordle::new();
    for answer in GAMES
        .split_whitespace()
        .take(args.max.unwrap_or(usize::MAX))
    {
        let guesser = wordle::WordleSolver::new();
        if let Some(score) = wordle.play(
            answer
                .as_bytes()
                .try_into()
                .expect("Answer must be 5 characters"),
            guesser,
        ) {
            println!("guesed {} in {}", answer, score);
        } else {
            eprintln!("failed to guess");
        }
    }
}
