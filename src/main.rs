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
    /// Flag to run fullscreen or not
    #[clap(short, long)]
    fullscreen: bool,

    /// Flag to print framerate or not
    #[clap(short = 'F', long)]
    fps: bool,
}

fn main() {
    let cli = Cli::parse();
    run(cli.fullscreen, cli.fps);
}
