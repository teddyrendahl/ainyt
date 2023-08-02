use crate::{Correctness, Guess, Guesser};

pub struct Naive {}

impl Naive {
    pub fn new() -> Self {
        Self {}
    }
}
impl Guesser for Naive {
    fn guess(&self, history: &[Guess]) -> String {
        todo!()
    }
}
