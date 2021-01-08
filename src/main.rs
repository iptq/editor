#![feature(vec_into_raw_parts)]

#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate log;
extern crate bass_sys as bass;

mod audio;
mod game;
mod math;
mod skin;
mod slider_render;

use std::env;

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

    let cb = ContextBuilder::new("osu_editor", "ggez")
        .add_resource_path("skin")
        .window_setup(WindowSetup::default().title("OSU editor"))
        .window_mode(WindowMode::default().dimensions(1024.0, 768.0));

    let (ctx, event_loop) = &mut cb.build()?;
    let mut game = Game::new()?;
    let path = env::args().nth(1).unwrap();
    game.load_beatmap(path)?;
    event::run(ctx, event_loop, &mut game)?;

    Ok(())
}
