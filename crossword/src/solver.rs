use std::collections::{HashMap, VecDeque};

use chatgpt::prelude::ChatGPT;
use thirtyfour::prelude::WebDriverResult;

use crate::{web::MiniCrosswordWebDriver, Clue, Grid};

/// Generate a ChatGPT prompt for a given Clue
fn prompt_for_clue(clue: &Clue, grid: &Grid, clues: &[Clue]) -> String {
    let current_answer = grid.answer_for(clue);
    format!(
        "Determine the answer to the following crossword clue.
        <text>
        {}
        </text>
        The answer is {} letters long and has the pattern {}. Respond with just the answer, no other text. Do not include punctuation or hyphens.
        For reference the other clues are included below.
        <text>
        {}
        </text>
        ",
        clue.text,
        current_answer.len(),
        current_answer,
        clues.iter().map(|c| format!("{}-{:?}: {} \n", c.number, c.direction, c.text)).collect::<String>()
    )
}
pub struct GPTSolver {
    gpt: ChatGPT,
    cache: HashMap<String, Option<String>>,
}

impl GPTSolver {
    pub fn new(api_key: String) -> chatgpt::Result<Self> {
        Ok(Self {
            gpt: ChatGPT::new(api_key)?,
            cache: HashMap::new(),
        })
    }

    /// Generate an answer for a provided Clue
    async fn solve_clue(
        &mut self,
        clue: &Clue,
        grid: &Grid,
        clues: &[Clue],
    ) -> chatgpt::Result<Option<String>> {
        let prompt = prompt_for_clue(clue, grid, clues);
        println!("{}", prompt);
        // If we've asked this before don't bother asking again
        if let Some(ans) = self.cache.get(&prompt) {
            println!("Cached answer {:?}", ans);
            Ok(ans.clone())
        } else {
            // Ask ChatGPT
            let answer = self
                .gpt
                .send_message(prompt.clone())
                .await?
                .message()
                .content
                .clone()
                // This is frustrating we have to do this... we asked GPT not to include them.
                .replace('_', "");
            println!("{}", answer);
            // Check this answer could plausibly be entered by verifying the length of the response
            if answer.len() == grid.cells_for_clue(clue).len() {
                self.cache.insert(prompt, Some(answer.clone()));
                Ok(Some(answer))
            } else {
                Ok(None)
            }
        }
    }

    pub async fn solve(
        &mut self,
        grid: &mut Grid,
        clues: Vec<Clue>,
        driver: &MiniCrosswordWebDriver,
    ) -> WebDriverResult<bool> {
        for i in 0..clues.len() {
            // Attempt to solve the Grid starting at a new Clue each attempt. This can
            // help us escape situations where ChatGPT gives us a wrong but feasible
            // answer for the first clues in the puzzle
            if self.solve_grid(clues.clone(), grid, driver, i).await? {
                return Ok(true);
            }
            grid.clear();
            driver.clear(grid).await?;
        }
        Ok(false)
    }
    /// Attempt to solve a Grid until we get stuck
    async fn solve_grid(
        &mut self,
        clues: Vec<Clue>,
        grid: &Grid,
        driver: &MiniCrosswordWebDriver,
        start_clue_idx: usize,
    ) -> WebDriverResult<bool> {
        let mut next_clues = VecDeque::from(clues.clone());
        next_clues.rotate_left(start_clue_idx);
        while let Some(clue) = next_clues.pop_front() {
            let cells = grid.cells_for_clue(&clue);
            // Do not attempt to solve an already completed answer
            if cells.iter().all(|(_, f)| f.value().is_some()) {
                continue;
            }
            // Request a new answer for the Clue from the ChatGPT
            let Some(answer) = self.solve_clue(&clue, grid, &clues).await.expect("Error with ChatGPT API") else {
                    continue;
                };

            // If the answer fits in our current Grid continue on
            if cells
                .iter()
                .zip(answer.chars())
                .all(|((_, f), c)| f.value().is_none() || f.value() == Some(c))
            {
                // Enter our new answer into the grid
                driver
                    .enter_answer(&clue, &answer, grid)
                    .await
                    .expect("Failed to enter answer into Grid");
                grid.enter_answer(&clue, answer);
                grid.show();
                // Add any crosses to the front of our queue to try next. They have new information
                // for us to send to ChatGPT
                for cross in grid.crosses(&clue, &clues) {
                    next_clues.push_front(cross)
                }
            }
            // TODO: Save potential solutions with conflicting crosses removed as a potential
            //       seed for more attempts
        }
        // Exit condition when all cells are filled
        if driver
            .is_complete()
            .await
            .expect("Failed to search for exit dialog")
        {
            println!("Puzzle complete!");
            Ok(true)
        } else {
            // We probably have the pop-up telling us to keep trying... let's do that.
            driver
                .maybe_keep_trying()
                .await
                .expect("Failed to hit the keep trying button");
            Ok(false)
        }
    }
}
