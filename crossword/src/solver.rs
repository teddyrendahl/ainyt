use std::{
    collections::{HashMap, VecDeque},
    fmt::Write,
    time::Duration,
};

use async_trait::async_trait;
use chatgpt::prelude::ChatGPT;

use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
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
        entries.iter().fold(String::new(),
         |mut output, entry| {
            let _  = writeln!(&mut output, "{}-{:?}: {} ", entry.clue().number, entry.clue().direction, entry.clue().text);
             output})
    )
}

pub enum APIKey {
    OpenAI(String),
    Cohere(String),
}

pub struct LLMSolver {
    llm: Box<dyn LLMModel>,
    cache: HashMap<String, Option<String>>,
}

impl LLMSolver {
    pub fn new(api_key: APIKey) -> chatgpt::Result<Self> {
        Ok(Self {
            llm: match api_key {
                APIKey::OpenAI(key) => {
                    Box::new(OpenAI::new(key).expect("Failed to connect to ChatGPT"))
                }
                APIKey::Cohere(key) => Box::new(Cohere::new(key)),
            },
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
        println!("{}-{:?}", entry.clue().number, entry.clue().direction);
        // If we've asked this before don't bother asking again
        if let Some(ans) = self.cache.get(&prompt) {
            println!("Cached answer {:?}", ans);
            Ok(ans.clone())
        } else {
            // Ask ChatGPT
            let answer = self
                .llm
                .chat(prompt.clone())
                .await
                .to_ascii_uppercase()
                // This is frustrating we have to do this... we asked not to include them.
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
            let Some(answer) = self
                .solve_clue(&entry, &entries)
                .await
                .expect("Error with ChatGPT API")
            else {
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

#[async_trait]
trait LLMModel {
    async fn chat(&self, message: String) -> String;
}

struct Cohere {
    client: reqwest::Client,
    key: String,
}

impl Cohere {
    fn new(key: String) -> Self {
        Cohere {
            client: reqwest::Client::builder().build().unwrap(),
            key,
        }
    }
}

#[derive(Deserialize)]
struct CohereChatResponse {
    text: String,
}

#[derive(Serialize)]
struct CohereRequest {
    message: String,
}

#[async_trait]
impl LLMModel for Cohere {
    async fn chat(&self, message: String) -> String {
        let response = self
            .client
            .post("https://api.cohere.ai/v1/chat")
            .header("Authorization", &format!("Bearer {}", self.key))
            .header("Content-Type", "application/json")
            .body(
                serde_json::to_string(&CohereRequest {
                    message: message.clone(),
                })
                .expect("Failed to serialize request"),
            )
            .send()
            .await
            .expect("Failed to connect to Cohere");

        // 5 requests per minute on the trial license
        if response.status() == StatusCode::TOO_MANY_REQUESTS {
            tokio::time::sleep(Duration::from_secs(1)).await;
            return self.chat(message).await;
        }
        response
            .error_for_status()
            .expect("Unexpected status")
            .json::<CohereChatResponse>()
            .await
            .expect("Failed to parse CohereResponse")
            .text
    }
}

struct OpenAI {
    gpt: ChatGPT,
}

impl OpenAI {
    fn new(api_key: String) -> chatgpt::Result<Self> {
        Ok(Self {
            gpt: ChatGPT::new(api_key)?,
        })
    }
}

#[async_trait]
impl LLMModel for OpenAI {
    async fn chat(&self, message: String) -> String {
        self.gpt
            .send_message(message)
            .await
            .expect("Failed to reach ChatGPT")
            .message()
            .content
            .clone()
    }
}
