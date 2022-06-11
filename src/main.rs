use clap::Parser;
use window::run;

mod camera;
mod core;
mod cube;
mod scene;
mod texture;
mod window;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// Whether to run fullscreen or not
    #[clap(short, long)]
    fullscreen: bool,
}

fn main() {
    let cli = Cli::parse();
    run(cli.fullscreen);
}
