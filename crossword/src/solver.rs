use chatgpt::prelude::ChatGPT;

use crate::{web::MiniCrosswordWebDriver, Clue, Grid};

/// Generate a ChatGPT prompt for a given Clue
fn prompt_for_clue<const W: usize, const H: usize>(clue: &Clue, grid: &Grid<W, H>) -> String {
    let current_answer = grid.answer_for(clue);
    format!(
        "What is the answer to the crossword clue \"{}\". \
             The answer is {} letters long with the pattern {}? \
             Respond with just the answer",
        clue.text,
        current_answer.len(),
        current_answer
    )
}
pub struct GPTSolver(ChatGPT);

impl GPTSolver {
    pub fn new(api_key: String) -> chatgpt::Result<Self> {
        Ok(Self(ChatGPT::new(api_key)?))
    }
    pub async fn solve<const W: usize, const H: usize>(
        &self,
        grid: &Grid<W, H>,
        clues: &[Clue],
        driver: &MiniCrosswordWebDriver,
    ) -> chatgpt::Result<()> {
        let mut conversation = self.0.new_conversation();
        for clue in clues {
            let prompt = prompt_for_clue(clue, grid);
            let cells = grid.cells_for_clue(clue);
            println!("{}", prompt);
            let answer = conversation
                .send_message(prompt)
                .await?
                .message()
                .content
                .clone();
            println!("{}", answer);
            if cells.len() != answer.len() {
                println!(
                    "Answer {} does not match clue length of {}",
                    answer,
                    cells.len()
                )
            } else {
                driver
                    .enter_answer(clue, &answer)
                    .await
                    .expect("Failed to enter answer into Grid");
                grid.enter_answer(clue, answer);
            }
        }
        Ok(())
    }
}
