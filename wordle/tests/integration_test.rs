use wordle::Wordle;

const GAMES: &str = include_str!("../answers.txt");

#[test]
fn complete_solves() {
    let wordle = Wordle::new();
    for answer in GAMES.split_whitespace().take(250) {
        let guesser = wordle::WordleSolver::new();
        assert!(wordle
            .play(answer.as_bytes().try_into().unwrap(), guesser)
            .is_some())
    }
}
