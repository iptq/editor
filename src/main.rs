#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate log;
extern crate bass_sys as bass;

mod audio;
mod game;

use anyhow::Result;
use ggez::{
    conf::{WindowMode, WindowSetup},
    event, ContextBuilder,
};

use crate::game::Game;

fn main() -> Result<()> {
    stderrlog::new()
        .module("editor")
        .module("bass_sys")
        .verbosity(2)
        .init()
        .unwrap();

    let cb = ContextBuilder::new("super_simple", "ggez")
        .window_setup(WindowSetup::default().title("OSU editor"))
        .window_mode(WindowMode::default().dimensions(1024.0, 768.0));

    let (ctx, event_loop) = &mut cb.build()?;
    let mut game = Game::new()?;
    game.load_beatmap("happy-time/Nanamori-chu  Goraku-bu - Happy Time wa Owaranai (Cut Ver.) (-Keitaro) [Osu's Expert].osu")?;
    event::run(ctx, event_loop, &mut game)?;

    Ok(())
}
