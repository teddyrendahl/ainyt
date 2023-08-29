use clap::Parser;
use crossword::{solver::GPTSolver, web::MiniCrosswordWebDriver, Grid};

#[derive(Parser)]
struct Opts {
    // Path to the Chrome binary. The 'thirtyfour' library will attempt to
    // find the binary itself, but certain installations may require this
    // to be passed explicitly.
    #[clap(short, long)]
    chrome_binary_path: Option<String>,
    // URL of running chromedriver application
    #[clap(short, long, default_value = "http://localhost:9515")]
    chromedriver_server_url: String,
    #[clap(short, long)]
    openapi_key: String,
}

#[tokio::main]
async fn main() {
    let opts: Opts = Opts::parse();
    let driver = MiniCrosswordWebDriver::create(
        &opts.chromedriver_server_url,
        opts.chrome_binary_path.as_deref(),
    )
    .await
    .expect("Failed to create WebDriver");
    let solver = GPTSolver::new(opts.openapi_key).expect("Failed to load GPTSolver");
    let puzzle = driver
        .get_puzzle()
        .await
        .expect("Failed to get Puzzle information");
    let grid = Grid::<5, 5>::from(&puzzle);
    driver.enter_answer(&puzzle.clues[0], "calf").await.unwrap();
    // Solve
    solver
        .solve(&grid, &puzzle.clues, &driver)
        .await
        .expect("Failed to solve Crossword puzzle!");
    // Verify answers for clues
    for clue in puzzle.clues {
        let attempted_answer = grid.answer_for(&clue);
        if clue.answer.as_ref().unwrap_or(&attempted_answer) != &attempted_answer {
            panic!(
                "{} does not match expected answer {}",
                attempted_answer,
                clue.answer.unwrap_or_default(),
            )
        }
    }
}
