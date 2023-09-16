use clap::Parser;
use crossword::{solver::GPTSolver, web::MiniWebPuzzle};

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
    let puzzle = MiniWebPuzzle::new(
        &opts.chromedriver_server_url,
        opts.chrome_binary_path.as_deref(),
    )
    .await
    .expect("Failed to read Puzzle information");
    let mut solver = GPTSolver::new(opts.openapi_key).expect("Failed to load GPTSolver");
    if solver
        .solve(&puzzle)
        .await
        .expect("Failed to solve Crossword puzzle!")
    {
        println!("Successfully solved Puzzle!")
    } else {
        println!("Failed to solve Puzzle!")
    }
}
