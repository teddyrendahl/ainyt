use clap::{clap_derive::ArgEnum, Parser};
use wordle::{Guesser, Wordle};

const GAMES: &str = include_str!("../answers.txt");

#[derive(Parser)]
struct Opt {
    #[clap(short, long, arg_enum)]
    implementation: Implementation,
    #[clap(short, long)]
    max: Option<usize>,
}

#[derive(ArgEnum, Debug, Clone, Copy)]
enum Implementation {
    Naive,
    Allocs,
    VecRemain,
}

fn main() {
    let args = Opt::parse();
    match args.implementation {
        Implementation::Naive => play(wordle::algorithms::Naive::new, args.max),
        Implementation::Allocs => play(wordle::algorithms::Allocs::new, args.max),
        Implementation::VecRemain => play(wordle::algorithms::VecRemain::new, args.max)
    }
}

fn play<G>(mut mk: impl FnMut() -> G, max: Option<usize>)
where
    G: Guesser,
{
    let wordle = Wordle::new();
    for answer in GAMES.split_whitespace().take(max.unwrap_or(usize::MAX)) {
        let guesser = (mk)();
        if let Some(score) = wordle.play(answer, guesser) {
            println!("guesed {} in {}", answer, score);
        } else {
            eprintln!("failed to guess");
        }
    }
}
