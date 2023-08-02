use wordle::Wordle;

const GAMES: &str = include_str!("../answers.txt");

fn main() {
    let wordle = Wordle::new();
    for answer in GAMES.split_whitespace() {
        let guesser = wordle::algorithms::Naive::new();
        if let Some(score) = wordle.play(answer, guesser) {
            println!("guesed {} in {}", answer, score);
        } else {
            eprintln!("failed to guess");
        }
    }
}
