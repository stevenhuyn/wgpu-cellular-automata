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

    /// Width of simulation grid
    #[clap(short, long)]
    grid_width: Option<u32>,
}

fn main() {
    let cli = Cli::parse();
    let grid_width = cli.grid_width.unwrap_or(30);
    run(cli.fullscreen, cli.fps, grid_width);
}
