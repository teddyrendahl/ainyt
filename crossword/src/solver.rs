use std::time::Duration;

use chatgpt::prelude::ChatGPT;

use crate::{web::MiniCrosswordWebDriver, Clue, Grid};

/// Generate a ChatGPT prompt for a given Clue
fn prompt_for_clue(clue: &Clue, grid: &Grid) -> String {
    let current_answer = grid.answer_for(clue);
    format!(
        "Determine the answer to the following crossword clue.
        <text>
        {}
        </text>
        The answer is {} letters long. Respond with just the answer, no other text. Do not include punctuation or hyphens. ",
        clue.text,
        current_answer.len(),
        // current_answer
    )
}
pub struct GPTSolver(ChatGPT);

impl GPTSolver {
    pub fn new(api_key: String) -> chatgpt::Result<Self> {
        Ok(Self(ChatGPT::new(api_key)?))
    }
    pub async fn solve(
        &self,
        grid: &Grid,
        clues: &[Clue],
        driver: &MiniCrosswordWebDriver,
    ) -> chatgpt::Result<()> {
        loop {
            for clue in clues {
                let cells = grid.cells_for_clue(clue);
                // Otherwise create a prompt and ask ChatGPT for an answer
                let prompt = prompt_for_clue(clue, grid);
                println!("{}", prompt);
                let answer = self
                    .0
                    .send_message(prompt)
                    .await?
                    .message()
                    .content
                    .clone()
                    // This is frustrating we have to do this... we asked GPT not to include
                    .replace('_', "");
                println!("{}", answer);
                // ChatGPT sent us an answer with the incorrect amount of characters just move on
                if cells.len() != answer.len() {
                    println!(
                        "Answer {} does not match clue length of {}",
                        answer,
                        cells.len()
                    )
                } else {
                    // Sometimes Chat-GPT gives an answer of the correct length, but not the correct
                    // filter. We take this as a sign that we may have a previous answer incorrect (and this
                    // new one is correct. So we enter this and clear out the old one)
                    // for ((pos, fill), c) in cells.iter().zip(answer.chars()) {
                    //     if fill.value().is_some_and(|v| v != c) {
                    //         let cross_direction = match clue.direction {
                    //             Direction::Across => Direction::Down,
                    //             Direction::Down => Direction::Across,
                    //         };
                    //         // Find the crossing clue and clear it
                    //         for cross in clues.iter() {
                    //             // Clue is crossing if it contains the position with the opposite direction
                    //             if cross.direction == cross_direction
                    //                 && grid
                    //                     .cells_for_clue(cross)
                    //                     .iter()
                    //                     .map(|(p, _)| p)
                    //                     .contains(&pos)
                    //             {
                    //                 driver
                    //                     .clear_answer(cross, grid)
                    //                     .await
                    //                     .expect("Failed to clear answer");
                    //                 grid.clear_answer(cross);
                    //             }
                    //         }
                    //     }
                    // }
                    // Enter our new answer into the grid
                    driver
                        .enter_answer(clue, &answer, grid)
                        .await
                        .expect("Failed to enter answer into Grid");
                    grid.enter_answer(clue, answer);
                    grid.show();
                }
                if grid.filled() {
                    tokio::time::sleep(Duration::from_millis(500)).await;
                    // Exit condition when all cells are filled
                    if driver
                        .is_complete()
                        .await
                        .expect("Failed to search for exit dialog")
                    {
                        println!("Puzzle complete!");
                        return Ok(());
                    }

                    // Make sure we don't have the keep trying box up
                    driver
                        .maybe_keep_trying()
                        .await
                        .expect("Failed to hit the keep trying button");
                }
            }
        }
    }
}
