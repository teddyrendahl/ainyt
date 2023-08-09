use std::time::Duration;

use thirtyfour::{prelude::WebDriverResult, By, ChromeCapabilities, WebDriver, WebElement};

use crate::Correctness;

static WORDLE_URL: &str = "https://www.nytimes.com/games/wordle/index.html";
static WORLD_GAME_CSS_ID: &str = "wordle-app-game";
static TILE_CSS: &str = "div[aria-roledescription=\"tile\"]";

pub struct WordleWebDriver(WebDriver);

impl WordleWebDriver {
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
        driver.goto(WORDLE_URL).await?;
        for button_cls in [
            "purr-blocker-card__button",
            "Welcome-module_buttonContainer__K4GEw .Welcome-module_button__ZG0Zh",
            "Modal-module_closeIcon__TcEKb",
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

    /// Get the main game WebElement from the page
    async fn get_game(&self) -> WebDriverResult<WebElement> {
        let game_app = self.0.find(By::Id(WORLD_GAME_CSS_ID)).await?;
        Ok(game_app)
    }

    pub async fn guess(&self, answer: &str, row: usize) -> WebDriverResult<[Correctness; 5]> {
        self.enter_answer(answer).await?;
        tokio::time::sleep(Duration::from_secs(2)).await;
        self.get_mask(row).await
    }

    /// Enter an answer into the Wordle Grid
    async fn enter_answer(&self, answer: &str) -> WebDriverResult<()> {
        let game = self.get_game().await?;
        for char in answer.to_ascii_lowercase().chars() {
            game.find(By::Css(&format!("button[data-key=\"{char}\"]")))
                .await?
                .click()
                .await?;
        }
        // TODO: Use a wait instead of a sleep
        tokio::time::sleep(Duration::from_millis(500)).await;
        game.find(By::Css("button[data-key=\"↵\"]"))
            .await?
            .click()
            .await?;
        Ok(())
    }

    /// Get the mask produced by the last Nth guess
    async fn get_mask(&self, row: usize) -> WebDriverResult<[Correctness; 5]> {
        let game = self.get_game().await?;
        let row = game
            .find(By::Css(&format!("div[aria-label=\"Row {row}\"")))
            .await?;
        let mut mask = vec![];
        for c in row.find_all(By::Css(TILE_CSS)).await? {
            match c.attr("data-state").await?.unwrap().as_str() {
                "absent" => mask.push(Correctness::Wrong),
                "present" => mask.push(Correctness::Misplaced),
                "correct" => mask.push(Correctness::Correct),
                _ => panic!("Unrecognized tile state"),
            }
        }
        Ok(mask.try_into().unwrap())
    }
}
