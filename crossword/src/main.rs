use clap::{Args, Parser};
use crossword::{
    solver::{APIKey, LLMSolver},
    web::MiniWebPuzzle,
};

#[derive(Parser)]
struct Opts {
    // Path to the Chrome binary. The 'thirtyfour' library will attempt to
    // find the binary itself, but certain installations may require this
    // to be passed explicitly.
    #[clap(long)]
    chrome_binary_path: Option<String>,
    // URL of running chromedriver application
    #[clap(long, default_value = "http://localhost:9515")]
    chromedriver_server_url: String,
    #[clap(flatten)]
    key: KeyOpts,
}

#[derive(Args)]
#[group(required = true, multiple = false)]
struct KeyOpts {
    #[clap(long)]
    openai: Option<String>,
    #[clap(long)]
    cohere: Option<String>,
}

impl From<KeyOpts> for APIKey {
    fn from(opts: KeyOpts) -> Self {
        if let Some(k) = opts.openai {
            APIKey::OpenAI(k)
        } else if let Some(k) = opts.cohere {
            APIKey::Cohere(k)
        } else {
            panic!("No key provided")
        }
    }
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
    let mut solver = LLMSolver::new(opts.key.into()).expect("Failed to load GPTSolver");
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
