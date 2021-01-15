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
mod utils;

use std::path::PathBuf;

use anyhow::Result;
use ggez::{
    conf::{WindowMode, WindowSetup},
    event, graphics, ContextBuilder,
};
use imgui::{Context as ImguiContext, FontConfig, FontGlyphRanges, FontSource};
use imgui_gfx_renderer::{Renderer, Shaders};
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use structopt::StructOpt;

use crate::game::Game;

type ColorFormat = gfx::format::Rgba8;
type DepthFormat = gfx::format::DepthStencil;

type Device = gfx_device_gl::Device;
type Factory = gfx_device_gl::Factory;
type Resources = gfx_device_gl::Resources;

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
        .module("libosu::spline")
        .verbosity(opt.verbose)
        .show_module_names(true)
        .init()
        .unwrap();

    let cb = ContextBuilder::new("osu_editor", "ggez")
        .add_resource_path("skin")
        .window_setup(WindowSetup::default().title("OSU editor"))
        .window_mode(WindowMode::default().dimensions(1024.0, 768.0));

    let (mut ctx, event_loop) = cb.build()?;

    let mut imgui = ImguiContext::create();
    let mut imgui_platform = WinitPlatform::init(&mut imgui);
    {
        let window = graphics::window(&ctx);
        imgui_platform.attach_window(imgui.io_mut(), window, HiDpiMode::Default);
    }

    let hidpi_factor = imgui_platform.hidpi_factor();
    let font_size = (13.0 * hidpi_factor) as f32;
    imgui.fonts().add_font(&[
        FontSource::DefaultFontData {
            config: Some(FontConfig {
                size_pixels: font_size,
                ..FontConfig::default()
            }),
        },
        FontSource::TtfData {
            data: include_bytes!("../font/Roboto-Regular.ttf"),
            size_pixels: font_size,
            config: Some(FontConfig {
                rasterizer_multiply: 1.75,
                glyph_ranges: FontGlyphRanges::default(),
                ..FontConfig::default()
            }),
        },
    ]);
    imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;

    let (factory, _, _, _, _) = graphics::gfx_objects(&mut ctx);
    let renderer = Renderer::init(&mut imgui, factory, Shaders::GlSl130)?;

    let mut game = Game::new(imgui, imgui_platform, renderer)?;
    game.skin.load_all(&mut ctx)?;

    if let Some(path) = opt.path {
        game.load_beatmap(&mut ctx, path)?;
    }

    if let Some(start_time) = opt.start_time {
        game.jump_to_time(start_time)?;
    }

    event::run(ctx, event_loop, game)
}
