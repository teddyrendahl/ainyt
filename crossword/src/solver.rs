use std::collections::{HashMap, VecDeque};

use chatgpt::prelude::ChatGPT;
use thirtyfour::prelude::WebDriverError;

use crate::{
    web::{fork_entries, MiniWebPuzzle, WebEntry},
    Clue, InMemoryEntry,
};

/// Generate a ChatGPT prompt for a given Clue
async fn prompt_for_clue(entry: &WebEntry, entries: &[WebEntry]) -> String {
    let current_answer = entry.value().await;
    format!(
        "Determine the answer to the following crossword clue.
        <text>
        {}
        </text>
        The answer is {} letters long and may match the pattern {}. Respond with just the answer, no other text. Do not include punctuation or hyphens.
        For reference the other clues are included below.
        <text>
        {}
        </text>
        ",
        entry.clue().text,
        current_answer.len(),
        current_answer,
        entries.iter().map(|entry| format!("{}-{:?}: {} \n", entry.clue().number, entry.clue().direction, entry.clue().text)).collect::<String>()
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
        entry: &WebEntry,
        entries: &[WebEntry],
    ) -> chatgpt::Result<Option<String>> {
        let prompt = prompt_for_clue(entry, entries).await;
        // println!("{}", prompt);
        println!("{}-{:?}", entry.clue().number, entry.clue().direction);
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
                .to_ascii_uppercase()
                // This is frustrating we have to do this... we asked GPT not to include them.
                .replace('_', "");
            println!("{}", answer);
            // Check this answer could plausibly be entered by verifying the length of the response
            if answer.len() == entry.length().await {
                self.cache.insert(prompt, Some(answer.clone()));
                Ok(Some(answer))
            } else {
                Ok(None)
            }
        }
    }

    pub async fn solve(&mut self, puzzle: &MiniWebPuzzle) -> Result<bool, WebDriverError> {
        let entries = puzzle.generate_entries().await?;

        let mut checkpoints = VecDeque::from([HashMap::<Clue, InMemoryEntry>::new()]);
        while let Some(state) = checkpoints.pop_front() {
            // Set the state back to the checkpoint
            for entry in entries.iter() {
                if let Some(cached_entry) = state.get(&entry.clue()) {
                    entry.write_chars(cached_entry.chars().await).await?;
                }
            }
            match self.solve_grid(entries.clone(), puzzle).await {
                Ok(_) => return Ok(true),
                Err(GridSolveError::WebDriverError(e)) => return Err(e),
                Err(GridSolveError::FailedToSolve(cps)) => {
                    for checkpoint in cps {
                        checkpoints.push_front(checkpoint);
                    }
                }
            }
        }
        Ok(false)
    }
    /// Attempt to solve a Grid until we get stuck
    async fn solve_grid(
        &mut self,
        entries: Vec<WebEntry>,
        puzzle: &MiniWebPuzzle,
        // start_entry_idx: usize,
    ) -> Result<(), GridSolveError> {
        let mut next_entries = VecDeque::from(entries.clone());
        // next_entries.rotate_left(start_entry_idx);
        let mut checkpoints = vec![];
        while let Some(entry) = next_entries.pop_front() {
            // Do not attempt to solve an already completed answer
            if entry.filled().await {
                continue;
            }
            // Request a new answer for the Clue from the ChatGPT
            let Some(answer) = self.solve_clue(&entry,  &entries).await.expect("Error with ChatGPT API") else {
                    continue;
                };
            // If the answer fits in our current Grid continue on
            if entry.fits(&answer).await {
                // Enter our new answer into the grid
                entry.write(answer).await?;
                // Add any crosses to the front of our queue to try next. They have new information
                // for us to send to ChatGPT
                for cross in entry.crossing_entries(entries.clone()) {
                    next_entries.push_front(cross)
                }
            } else {
                // We want to capture the state of the system if we used this answer
                // and cleared out any other answers that disagree. This is our "backtrack"
                // that gives us a chance to clear out old bad answers
                println!("Capturing backtrack to enter {} in grid", answer);
                // Capture the current state of all the cells.
                let state = fork_entries(&entries).await;
                // Find crossing clues that are creating the conflict and clear them out
                for cross in entry.conflicting_entries(&answer, &entries).await {
                    state
                        .get(&cross.clue())
                        .expect("Cross not in state!")
                        .clear()
                        .await
                }
                // Write the new answer in to the old state
                state
                    .get(&entry.clue())
                    .expect("Entry not in state!")
                    .write(answer)
                    .await;
                checkpoints.push(state);
            }
        }
        if puzzle.verify_entries(&entries).await? {
            Ok(())
        } else {
            // We reverse here because we want to try states we discovered later in the solve first
            checkpoints.reverse();
            Err(GridSolveError::FailedToSolve(checkpoints))
        }
    }
}

enum GridSolveError {
    FailedToSolve(Vec<HashMap<Clue, InMemoryEntry>>),
    WebDriverError(WebDriverError),
}

impl From<WebDriverError> for GridSolveError {
    fn from(value: WebDriverError) -> Self {
        GridSolveError::WebDriverError(value)
    }
}
