# Wordle
An automated Wordle solver written In Rust

Inspired by two YouTube creators:

* [3Blue1Brown](https://www.youtube.com/watch?v=v68zYyaEmEA) - The description of the algorithm used in this codebase.
* [Jon Gjengset](https://www.youtube.com/watch?v=doFowk4xj7Q&t=14978s) - An implementation of the algorithm in Rust with discussions around reducing time complexity without threading.

# How To Use
Wordle depends on a running instance of `chromedriver`. It is not built as part of this repository. By default, it is expected that `chromedriver` is running locally on port `9515`,
but an alternate port can be passed in via the command line. If `chromedriver` is running then execute via:
```shell
$ cargo run --bin wordle --release
```

# How It Works
I strongly encourage you to watch [this](https://www.youtube.com/watch?v=v68zYyaEmEA) video which explains the algorithm, but in short, the goal is for each guess to provide the maximal possible "information" about our the target word. We can create an estimate for a single guess's "expected information" by looking at the probability for an event to occur, multiplied by the information that outcome would give us, totaled for every possible event. In information theory this is referred to as "entropy"

To illustrate this let's take an example guess like "apple". One possible outcome of this guess is only the first letter is correct, and the rest are not included in the answer. We can determine the probability of this by seeing how many words start with the letter "a" and do not contain "p",
"l", or "e". We can also determine the information that result would give us by looking at the `log2` of the 1/probability. By summing this calculation for every possible result of our guess, we get an "expected information" value for the guess of "apple". By selecting words that yield on average the maximal possible information, we can efficiently play the Wordle game. Intuitively, you can think of us trying to find the sweet spot between two forces; a very rare occurring event will tell us a lot about our word. For instance, if we guess a word where the letter "z" occurs we suddenly have a much more narrow set of possible words. On the other hand, the likelihood of that occuring is poor, so we'd expect to get good information from a guess with "z" less often. 

One other caveat is the official Wordle word set includes words that are unlikely to actually be the result of the puzzle posted in the New York Times. For instance, you'll notice words like "abcee" listed in the set. In an effort to discourage our algorithm from selecting these words we weight them by how common they are in the Google Books N-gram dataset. Using these we can get a fairly decent estimate for how common a specific word is. The Wordle dictionary and the corresponding counts are kept in the `dictionary.txt` file in this repository. 
