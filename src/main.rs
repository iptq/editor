#![windows_subsystem = "windows"]

#[macro_use]
extern crate anyhow;
#[allow(unused_macros, unused_imports)]
#[macro_use]
extern crate log;
extern crate bass_sys as bass;

mod audio;
mod beatmap;
mod game;
mod hitobject;
mod skin;

use std::path::PathBuf;

use anyhow::Result;
use ggez::{
    conf::{WindowMode, WindowSetup},
    event, ContextBuilder,
};
use structopt::StructOpt;

use crate::game::Game;

#[derive(StructOpt)]
struct Opt {
    #[structopt(short = "s")]
    start_time: Option<f64>,

    path: Option<PathBuf>,

    /// Verbose mode (-v, -vv, -vvv, etc)
    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    verbose: usize,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    stderrlog::new()
        .module("editor")
        .module("bass_sys")
        .verbosity(opt.verbose)
        .init()
        .unwrap();

    let cb = ContextBuilder::new("osu_editor", "ggez")
        .add_resource_path("skin")
        .window_setup(WindowSetup::default().title("OSU editor"))
        .window_mode(WindowMode::default().dimensions(1024.0, 768.0));

    let (mut ctx, mut event_loop) = cb.build()?;
    let mut game = Game::new()?;
    game.skin.load_all(&mut ctx)?;

    if let Some(path) = opt.path {
        game.load_beatmap(path)?;
    }

    if let Some(start_time) = opt.start_time {
        game.jump_to_time(start_time)?;
    }

    event::run(ctx, event_loop, game)
}
