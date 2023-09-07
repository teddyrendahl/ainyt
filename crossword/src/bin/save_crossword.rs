use std::{fs::File, io::BufWriter};

use clap::Parser;
use crossword::web::MiniCrosswordWebDriver;

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
    // Path to save crossword information
    #[clap(short, long)]
    output_path: String,
}

#[tokio::main]
/// Utility to save the current day's crossword to disk for later "replay"
///
/// This allows multiple days worth of crosswords to be tested against solving implementations
async fn main() {
    let opts: Opts = Opts::parse();
    let writer = BufWriter::new(File::create(opts.output_path).expect("Failed to create file"));
    let driver = MiniCrosswordWebDriver::create(
        &opts.chromedriver_server_url,
        opts.chrome_binary_path.as_deref(),
    )
    .await
    .expect("Failed to create WebDriver");
    let puzzle = driver
        .get_puzzle()
        .await
        .expect("Failed to get Puzzle information");
    serde_yaml::to_writer(writer, &puzzle).expect("Failed to write crossword information to file");
}
