use std::fs::File;
use std::io::Read;
use std::path::Path;

use anyhow::Result;
use ggez::{
    event::{EventHandler, KeyCode, KeyMods},
    graphics::{
        self, DrawMode, DrawParam, FillOptions, FilterMode, Mesh, Rect, StrokeOptions, Text, WHITE,
    },
    Context, GameError, GameResult,
};
use libosu::{Beatmap, HitObject, HitObjectKind};

use crate::audio::{AudioEngine, Sound};
use crate::slider_render::render_slider;

pub struct Game {
    is_playing: bool,
    audio_engine: AudioEngine,
    song: Option<Sound>,
    beatmap: Beatmap,
    hit_objects: Vec<HitObject>,
}

impl Game {
    pub fn new() -> Result<Game> {
        let audio_engine = AudioEngine::new()?;
        let beatmap = Beatmap::default();
        let hit_objects = Vec::new();

        Ok(Game {
            is_playing: false,
            audio_engine,
            beatmap,
            hit_objects,
            song: None,
        })
    }

    pub fn load_beatmap(&mut self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();

        let mut file = File::open(&path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        self.beatmap = Beatmap::from_osz(&contents)?;

        let dir = path.parent().unwrap();
        self.song = Sound::create(dir.join(&self.beatmap.audio_filename)).map(Some)?;

        Ok(())
    }

    pub fn toggle_playing(&mut self) {
        if self.is_playing {
            self.is_playing = false;
            self.audio_engine.pause(self.song.as_ref().unwrap());
        } else {
            self.is_playing = true;
            self.audio_engine.play(self.song.as_ref().unwrap());
        }
    }

    fn priv_draw(&mut self, ctx: &mut Context) -> Result<()> {
        // TODO: lol
        const EDITOR_SCREEN: Rect = Rect::new(112.0, 84.0, 800.0, 600.0);

        graphics::clear(ctx, [0.0, 0.0, 0.0, 1.0].into());

        let time = self.song.as_ref().unwrap().position()?;
        let text = Text::new(format!("time: {}", time).as_ref());
        graphics::queue_text(ctx, &text, [0.0, 0.0], Some(WHITE));
        graphics::draw_queued_text(ctx, DrawParam::default(), None, FilterMode::Linear)?;

        let mut visible_hitobjects = Vec::new();
        let approach_time = 0.75;
        for ho in self.beatmap.hit_objects.iter() {
            let ho_time = (ho.start_time.0 as f64) / 1000.0;
            if ho_time - approach_time < time && ho_time > time {
                visible_hitobjects.push(ho);
            }
        }

        let osupx_scale_x = EDITOR_SCREEN.w / 512.0;
        let osupx_scale_y = EDITOR_SCREEN.h / 384.0;
        let cs_osupx = 54.4 - 4.48 * self.beatmap.difficulty.circle_size;
        let cs_real = cs_osupx * osupx_scale_x;

        for ho in visible_hitobjects.iter() {
            let ho_time = (ho.start_time.0 as f64) / 1000.0;
            let pos = [
                EDITOR_SCREEN.x + osupx_scale_x * ho.pos.0 as f32,
                EDITOR_SCREEN.y + osupx_scale_y * ho.pos.1 as f32,
            ];

            if let HitObjectKind::Slider(_) = ho.kind {
                render_slider(ctx, EDITOR_SCREEN, &self.beatmap, ho)?;
            }

            let circ = Mesh::new_circle(
                ctx,
                DrawMode::Fill(FillOptions::default()),
                pos,
                cs_real,
                1.0,
                WHITE,
            )?;
            graphics::draw(ctx, &circ, DrawParam::default())?;

            let time_diff = ho_time - time;
            let approach_r = cs_real * (1.0 + 2.0 * time_diff as f32 / 0.75);
            let approach = Mesh::new_circle(
                ctx,
                DrawMode::Stroke(StrokeOptions::default()),
                pos,
                approach_r,
                1.0,
                WHITE,
            )?;
            graphics::draw(ctx, &approach, DrawParam::default())?;
        }

        graphics::present(ctx)?;
        Ok(())
    }
}

impl EventHandler for Game {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn key_up_event(&mut self, _: &mut Context, keycode: KeyCode, _: KeyMods) {
        if let KeyCode::Space = keycode {
            self.toggle_playing();
        }
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        if let Err(err) = self.priv_draw(ctx) {
            return Err(GameError::RenderError(err.to_string()));
        };
        Ok(())
    }
}
