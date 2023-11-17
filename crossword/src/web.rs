use futures::future::try_join_all;
use itertools::Itertools;
use regex::Regex;
use std::{collections::HashMap, time::Duration, vec};
use thirtyfour::{
    prelude::{WebDriverError, WebDriverResult},
    By, ChromeCapabilities, Key, WebDriver,
};

use crate::{positions_for_clue, Clue, Direction, InMemoryCell, InMemoryEntry, Position};

static MINI_URL: &str = "https://www.nytimes.com/crosswords/game/mini";
static SHADED_SQUARE_CLS: &str = "xwd__cell--block xwd__cell--nested";

static ENTRY_RATE_MS: u64 = 500;

async fn wait_on_entry() {
    tokio::time::sleep(Duration::from_millis(ENTRY_RATE_MS)).await
}

#[derive(Debug)]
struct WebGridInfo {
    width: usize,
    height: usize,
    clue_positions: HashMap<usize, Position>,
    shaded_squares: Vec<Position>,
}

#[derive(Clone)]
pub struct MiniCrosswordWebDriver(WebDriver);

impl MiniCrosswordWebDriver {
    pub async fn create(
        chromedriver_server_url: &str,
        binary_path: Option<&str>,
    ) -> WebDriverResult<Self> {
        let mut options = ChromeCapabilities::new();
        options.add_chrome_arg("--incognito")?;
        options.add_chrome_arg("--start-maximized")?;
        if let Some(p) = binary_path {
            options.set_binary(p)?;
        }
        let driver = WebDriver::new(chromedriver_server_url, options).await?;
        driver.goto(MINI_URL).await?;
        for button_cls in [
            // "purr-blocker-card__button", // Updated Terms of Service
            "xwd__modal--subtle-button", // Play With Free Account
        ] {
            wait_on_entry().await;
            driver
                .find(By::ClassName(button_cls))
                .await?
                .click()
                .await?;
            // TODO: Use a wait instead of a sleep
        }
        Ok(Self(driver))
    }

    // Get the information about the Grid itself
    async fn get_grid_info(&self) -> WebDriverResult<WebGridInfo> {
        // HTML id() attributes of the cell tells us the position in the crossword grid
        let re = Regex::new(r"cell-id-(\d*)").unwrap();
        // Get the size of the grid by seeing the number of unique X and Y values
        let cells = self.0.find_all(By::ClassName("xwd__cell")).await?;
        let columns = try_join_all(
            cells
                .iter()
                .map(|wbe| async move { wbe.find(By::Tag("RECT")).await?.css_value("x").await }),
        )
        .await?
        .into_iter()
        .unique()
        .count();

        let mut grid_info = WebGridInfo {
            width: columns,
            height: cells.len() / columns,
            clue_positions: HashMap::new(),
            shaded_squares: vec![],
        };

        // All cells have the same HTML class name
        for cell in cells.iter() {
            let r = cell.find(By::Css("rect[role=\"cell\"]")).await?;
            // Look at the id() of the rect inside the cell. If we can't interpret it
            // into a Position we are in trouble
            let position = Position::from_cell_id(
                re.captures_iter(&r.id().await?.expect("Cell missing ID"))
                    .map(|c| {
                        let (_, [s]) = c.extract();
                        s
                    })
                    .next()
                    .expect("Cell id does not match regex")
                    .parse()
                    .expect("Cell id is not valid usize"),
                columns,
            );
            // Shaded squares have a specific class name
            if r.class_name().await?.expect("Missing class name") == SHADED_SQUARE_CLS {
                grid_info.shaded_squares.push(position)
            }
            // Save cells with numbers so we can locate our clues
            else if !cell.text().await?.is_empty() {
                grid_info.clue_positions.insert(
                    cell.text()
                        .await?
                        .parse()
                        .expect("Cell text is not a clue number"),
                    position,
                );
            }
        }
        Ok(grid_info)
    }

    async fn get_clues(
        &self,
        clue_positions: HashMap<usize, Position>,
    ) -> WebDriverResult<Vec<Clue>> {
        let mut clues = vec![];
        for clue_list in self
            .0
            .find_all(By::ClassName("xwd__clue-list--wrapper"))
            .await?
        {
            // Determine the direction by the header at the top of the div
            let direction = match clue_list.find(By::Tag("H3")).await?.text().await?.as_str() {
                "ACROSS" => Direction::Across,
                "DOWN" => Direction::Down,
                s => panic!("Unexpected clue header {}", s),
            };
            // Scan through each clue in the list, grabbing number and text
            for clue in clue_list.find_all(By::Tag("LI")).await? {
                // Number and Text are in two different spans with different class names
                let number: usize = clue
                    .find(By::ClassName("xwd__clue--label"))
                    .await?
                    .text()
                    .await?
                    .parse()
                    .expect("Unable to turn clue label into number");
                let text = clue
                    .find(By::ClassName("xwd__clue--text"))
                    .await?
                    .text()
                    .await?;
                clues.push(Clue {
                    number,
                    direction,
                    text,
                    // Look where the clue is in the puzzle in our HashMap above
                    position: *clue_positions
                        .get(&number)
                        .expect("Clue not found in puzzle"),
                });
            }
        }
        Ok(clues)
    }

    async fn enter_in_cell(&self, cell_id: usize, text: impl AsRef<str>) -> WebDriverResult<()> {
        // Get cell based on position
        let cell = self.0.find(By::Id(&format!("cell-id-{}", cell_id))).await?;
        // Enter character into the cell
        self.0
            .action_chain()
            .click_element(&cell)
            .send_keys(text)
            .perform()
            .await?;
        // We wait here to not spam thru
        wait_on_entry().await;
        Ok(())
    }

    /// Whether the puzzle has been marked complete by NYT
    pub async fn is_complete(&self) -> WebDriverResult<bool> {
        Ok(!self
            .0
            .find_all(By::ClassName("xwd__congrats-modal--content"))
            .await?
            .is_empty())
    }

    /// Sometimes the keep trying box pops-up if we have filled the Grid incorrectly
    pub async fn maybe_keep_trying(&self) -> WebDriverResult<()> {
        for wbe in self
            .0
            .find_all(By::Css("button[aria-label=\"Keep trying\"]"))
            .await?
        {
            wbe.click().await?
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct WebCell {
    cell: InMemoryCell,
    driver: MiniCrosswordWebDriver,
    cell_id: usize,
}

impl WebCell {
    pub fn new(position: Position, driver: MiniCrosswordWebDriver, cell_id: usize) -> Self {
        Self {
            driver,
            cell: InMemoryCell::new(position, None),
            cell_id,
        }
    }

    async fn value(&self) -> Option<char> {
        self.cell.value().await
    }

    async fn write(&self, c: char) -> WebDriverResult<()> {
        if self.value().await != Some(c) {
            self.driver
                .enter_in_cell(self.cell_id, c.to_string())
                .await?;
            self.cell.write(c).await;
        }
        Ok(())
    }

    async fn clear(&self) -> WebDriverResult<()> {
        if self.value().await.is_some() {
            self.driver
                .enter_in_cell(self.cell_id, Key::Backspace.to_string())
                .await?;
            self.cell.clear().await;
        }
        Ok(())
    }

    fn position(&self) -> Position {
        self.cell.position
    }
}

#[derive(Clone)]
pub struct WebEntry {
    cells: Vec<WebCell>,
    clue: Clue,
}

impl WebEntry {
    pub fn clue(&self) -> Clue {
        self.clue.clone()
    }
    fn positions(&self) -> Vec<Position> {
        self.cells.iter().map(|c| c.position()).collect()
    }

    pub async fn chars(&self) -> Vec<Option<char>> {
        futures::future::join_all(self.cells.iter().map(|c| c.value()))
            .await
            .into_iter()
            .collect()
    }

    pub async fn write(&self, answer: String) -> Result<(), WebDriverError> {
        self.write_chars(answer.chars().map(Some).collect()).await
    }

    pub async fn write_chars(&self, chars: Vec<Option<char>>) -> Result<(), WebDriverError> {
        for (cell, char) in self.cells.iter().zip(chars) {
            if let Some(c) = char {
                cell.write(c).await?;
            } else {
                cell.clear().await?;
            }
        }
        Ok(())
    }

    pub async fn clear(&self) -> Result<(), WebDriverError> {
        for cell in self.cells.iter() {
            cell.clear().await?;
        }
        Ok(())
    }
    pub async fn value(&self) -> String {
        self.chars()
            .await
            .iter()
            .map(|c| c.unwrap_or('_'))
            .collect()
    }

    /// Length of the full Entry
    pub async fn length(&self) -> usize {
        self.chars().await.len()
    }

    /// Return a boolean if the provided answer fits with letters already populated in the Entry
    pub async fn fits(&self, ans: &str) -> bool {
        self.chars()
            .await
            .iter()
            .zip(ans.chars())
            .all(|(cell, c)| cell.is_none() || cell == &Some(c))
    }
    /// Whether the Entry has been fully populated
    pub async fn filled(&self) -> bool {
        self.chars().await.iter().all(|c| c.is_some())
    }

    // Return any Entries that conflict with entering the provided answer
    pub async fn conflicting_entries(&self, answer: &str, entries: &[Self]) -> Vec<Self> {
        let mut conflicts = vec![];
        for (c, ans) in self.cells.iter().zip(answer.chars()) {
            if c.value().await.is_some_and(|ch| ch != ans) {
                conflicts.push(entry_for_cell(
                    c.position(),
                    self.clue().direction.cross(),
                    entries,
                ));
            }
        }

        conflicts
    }

    // Get all crossing Entry values
    pub fn crossing_entries(&self, entries: Vec<Self>) -> Vec<Self> {
        let positions = self.positions();
        entries
            .into_iter()
            .filter_map(|e| {
                if e.clue().direction != self.clue().direction
                    && e.positions().iter().any(|p| positions.contains(p))
                {
                    Some(e)
                } else {
                    None
                }
            })
            .rev()
            .collect()
    }
}

pub struct MiniWebPuzzle {
    driver: MiniCrosswordWebDriver,
}

impl MiniWebPuzzle {
    pub async fn new(
        chromedriver_server_url: &str,
        binary_path: Option<&str>,
    ) -> WebDriverResult<Self> {
        Ok(MiniWebPuzzle {
            driver: MiniCrosswordWebDriver::create(chromedriver_server_url, binary_path).await?,
        })
    }

    pub async fn generate_entries(&self) -> WebDriverResult<Vec<WebEntry>> {
        let mut cells: HashMap<Position, WebCell> = HashMap::new();

        // Generate new Cells for the Grid, but we want to make sure only one Cell is created for
        // each position
        let grid_info = self.driver.get_grid_info().await?;
        Ok(self
            .driver
            .get_clues(grid_info.clue_positions)
            .await?
            .into_iter()
            .map(|clue| WebEntry {
                clue: clue.clone(),
                cells: positions_for_clue(
                    &clue,
                    grid_info.width,
                    grid_info.height,
                    &grid_info.shaded_squares,
                )
                .into_iter()
                .map(|p| {
                    cells.get(&p).cloned().unwrap_or_else(|| {
                        let cell = WebCell::new(
                            p,
                            self.driver.clone(),
                            p.row * grid_info.width + p.column,
                        );
                        cells.insert(p, cell.clone());
                        cell
                    })
                })
                .collect(),
            })
            .collect())
    }

    pub async fn verify_entries(&self, _entries: &[WebEntry]) -> Result<bool, WebDriverError> {
        if self.driver.is_complete().await? {
            Ok(true)
        } else {
            self.driver.maybe_keep_trying().await?;
            Ok(false)
        }
    }
}

pub fn entry_for_cell(position: Position, direction: Direction, entries: &[WebEntry]) -> WebEntry {
    entries
        .iter()
        .find(|e| e.positions().contains(&position) && e.clue().direction == direction)
        .unwrap()
        .clone()
}

// Fork a set of entries into ones that exist in Memory alone
pub async fn fork_entries(entries: &[WebEntry]) -> HashMap<Clue, InMemoryEntry> {
    let mut cells = HashMap::new();
    let mut forked_entries = HashMap::new();
    for entry in entries {
        for cell in entry.cells.iter() {
            if let std::collections::hash_map::Entry::Vacant(e) = cells.entry(cell.position()) {
                e.insert(InMemoryCell::new(cell.position(), cell.value().await));
            }
        }
        forked_entries.insert(
            entry.clue(),
            InMemoryEntry {
                cells: entry
                    .positions()
                    .iter()
                    .map(|p| cells.get(p).expect("Missing position!").clone())
                    .collect(),
            },
        );
    }
    forked_entries
}
