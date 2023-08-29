use std::{collections::HashMap, time::Duration};

use regex::Regex;
use thirtyfour::{prelude::WebDriverResult, By, ChromeCapabilities, WebDriver};

use crate::{Clue, Direction, Position, Puzzle};

static MINI_URL: &str = "https://www.nytimes.com/crosswords/game/mini";
static SHADED_SQUARE_CLS: &str = "xwd__cell--block xwd__cell--nested";

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
            "purr-blocker-card__button", // Updated Terms of Service
            "xwd__modal--subtle-button", // Play With Free Account
        ] {
            driver
                .find(By::ClassName(button_cls))
                .await?
                .click()
                .await?;
            // TODO: Use a wait instead of a sleep
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        Ok(Self(driver))
    }

    // Enter an answer into the Grid
    pub async fn enter_answer(&self, clue: &Clue, answer: &str) -> WebDriverResult<()> {
        let mut position = clue.position;
        for c in answer.chars() {
            // Get cell based on position
            let cell = self
                .0
                .find(By::Id(&format!(
                    "cell-id-{}",
                    position.row * 5 + position.column
                )))
                .await?;
            // Enter character into the cell
            self.0
                .action_chain()
                .click_element(&cell)
                .send_keys(c.to_string())
                .perform()
                .await?;

            // Go to the next cell in line
            match clue.direction {
                Direction::Across => position.column += 1,
                Direction::Down => position.row += 1,
            }
            // Wait before entering next letter
            tokio::time::sleep(Duration::from_millis(250)).await;
        }
        Ok(())
    }

    /// Read the HTML cell information to determine the Puzzle information
    pub async fn get_puzzle(&self) -> WebDriverResult<Puzzle> {
        let mut shaded_squares = Vec::new();
        let mut clue_positions: HashMap<usize, Position> = HashMap::new();
        let mut clues = vec![];
        // HTML id() attributes of the cell tells us the position in the crossword grid
        let re = Regex::new(r"cell-id-(\d*)").unwrap();
        // All cells have the same HTML class name
        for cell in self.0.find_all(By::ClassName("xwd__cell")).await? {
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
                5,
            );
            // Shaded squares have a specific class name
            if r.class_name().await?.expect("Missing class name") == SHADED_SQUARE_CLS {
                shaded_squares.push(position)
            }
            // Save cells with numbers so we can locate our clues
            else if !cell.text().await?.is_empty() {
                clue_positions.insert(
                    cell.text()
                        .await?
                        .parse()
                        .expect("Cell text is not a clue number"),
                    position,
                );
            }
        }
        // Get the clue descriptions
        // Across and Down clues are into two different sections with matching formats
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
                    answer: None,
                    text,
                    // Look where the clue is in the puzzle in our HashMap above
                    position: *clue_positions
                        .get(&number)
                        .expect("Clue not found in puzzle"),
                });
            }
        }
        Ok(Puzzle {
            width: 5,
            height: 5,
            shaded_squares,
            clues,
        })
    }
}
