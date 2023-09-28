# Mini Crossword
An automated solver of the NYT daily mini crossword

https://github.com/teddyrendahl/ainyt/assets/25753048/2893577e-b2f5-4c01-b702-9ebb9653c7ea

# How To Use
Wordle depends on a running instance of `chromedriver`. It is not built as part of this repository. By default, it is expected that `chromedriver` is running locally on port `9515`,
but an alternate port can be passed in via the command line. The code also uses the ChatGPT or Cohere API. Select one of them by providing a valid key (with credits) to either the `--openai` or `--cohere` CLI options.
```shell
$ cargo run --bin crossword --release -- --openai xxx
```

## How It Works
The solution relies on an LLM to provide answers for the various clues inside the puzzle. The grid is solved by alternating between across and down clues, ensuring that after the first clue we should have at least one known letter
for each query.

Often times the LLM will provide an answer that has a valid length, but does not match with the answers already entered into the grid. This could just be an erroneous answer, or it could mean that a prior answer is wrong. Both possibilities are explored by the algorithm, one that the answer is ignored and further clues are attempted, and one where the conflicting crosses are removed and the new answer is entered.
