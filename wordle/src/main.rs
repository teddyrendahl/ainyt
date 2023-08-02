use wordle::Wordle;

const GAMES: &str = include_str!("../answers.txt");

fn main() {
    let guesser = wordle::algorithms::Naive::new();
    let wordle = Wordle::new();
    for answer in GAMES.split_whitespace() {
        wordle.play(answer, &guesser);
    }
}
