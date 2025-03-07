use std::fs;

use anyhow::Context;
use clap::Parser;

use crate::render::Output;
use crate::scene::Scene;

// mod attract;
mod light;
mod material;
mod object;
mod ray;
mod render;
mod scene;

/// Ray-trace a scene simulating photon paths warped by gravity
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    scene: String,
    #[arg(short, long)]
    width: u32,
    #[arg(short, long)]
    height: u32,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let output = Output {
        width: cli.width,
        height: cli.height,
        escape: 5.0,
    };

    let scene_str = fs::read_to_string(&cli.scene).context("Read scene")?;
    let scene: Scene = serde_yaml::from_str(&scene_str).context("Parse scene")?;

    let pixels = render::render(&scene, &output);
    output.save_colors(&pixels, "output.png");
    output.save_normals(&pixels, "output.normals.png");

    Ok(())
}
