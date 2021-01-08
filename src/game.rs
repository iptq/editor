use std::fs::File;
use std::io::Read;
use std::path::Path;

use anyhow::Result;
use ggez::{
    event::{EventHandler, KeyCode, KeyMods},
    graphics::{
        self, Color, DrawMode, DrawParam, FillOptions, FilterMode, Mesh, Rect, StrokeOptions, Text,
        WHITE,
    },
    Context, GameError, GameResult,
};
use libosu::{Beatmap, HitObject, HitObjectKind, SpinnerInfo};

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

        let song = Sound::create(dir.join(&self.beatmap.audio_filename))?;
        song.set_position(113.0)?;
        self.song = Some(song);

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

        let playfield = Mesh::new_rectangle(
            ctx,
            DrawMode::Stroke(StrokeOptions::default()),
            EDITOR_SCREEN,
            Color::new(1.0, 1.0, 1.0, 0.5),
        )?;
        graphics::draw(ctx, &playfield, DrawParam::default())?;

        let time = self.song.as_ref().unwrap().position()?;
        let text = Text::new(format!("time: {}", time).as_ref());
        graphics::queue_text(ctx, &text, [0.0, 0.0], Some(WHITE));
        graphics::draw_queued_text(ctx, DrawParam::default(), None, FilterMode::Linear)?;

        struct DrawInfo<'a> {
            hit_object: &'a HitObject,
            opacity: f64,
            end_time: f64,
        }

        let mut visible_hitobjects = Vec::new();
        let preempt = (self.beatmap.difficulty.approach_preempt() as f64) / 1000.0;
        let fade_in = (self.beatmap.difficulty.approach_fade_time() as f64) / 1000.0;
        for ho in self.beatmap.hit_objects.iter() {
            let ho_time = (ho.start_time.0 as f64) / 1000.0;
            let end_time;
            let opacity = if time > ho_time - fade_in {
                1.0
            } else {
                // TODO: calculate ease
                (time - (ho_time - preempt)) / fade_in
            };
            match ho.kind {
                HitObjectKind::Circle => end_time = ho_time,
                HitObjectKind::Slider(_) => {
                    let duration = self.beatmap.get_slider_duration(ho).unwrap();
                    end_time = ho_time + duration / 1000.0;
                }
                HitObjectKind::Spinner(SpinnerInfo {
                    end_time: spinner_end,
                }) => end_time = (spinner_end.0 as f64) / 1000.0,
            };
            if ho_time - preempt < time && time < end_time {
                visible_hitobjects.push(DrawInfo {
                    hit_object: ho,
                    opacity,
                    end_time,
                });
            }
        }

        let osupx_scale_x = EDITOR_SCREEN.w / 512.0;
        let osupx_scale_y = EDITOR_SCREEN.h / 384.0;
        let cs_osupx = self.beatmap.difficulty.circle_size_osupx();
        let cs_real = cs_osupx * osupx_scale_x;

        for draw_info in visible_hitobjects.iter() {
            let ho = draw_info.hit_object;
            let ho_time = (ho.start_time.0 as f64) / 1000.0;
            let pos = [
                EDITOR_SCREEN.x + osupx_scale_x * ho.pos.0 as f32,
                EDITOR_SCREEN.y + osupx_scale_y * ho.pos.1 as f32,
            ];
            let color = graphics::Color::new(1.0, 1.0, 1.0, draw_info.opacity as f32);

            if let HitObjectKind::Slider(_) = ho.kind {
                let color = graphics::Color::new(1.0, 1.0, 1.0, 0.6 * draw_info.opacity as f32);
                render_slider(ctx, EDITOR_SCREEN, &self.beatmap, ho, color)?;
            }

            let circ = Mesh::new_circle(
                ctx,
                DrawMode::Fill(FillOptions::default()),
                pos,
                cs_real,
                1.0,
                color,
            )?;
            graphics::draw(ctx, &circ, DrawParam::default())?;

            if time < ho_time {
                let time_diff = ho_time - time;
                let approach_r = cs_real * (1.0 + 2.0 * time_diff as f32 / 0.75);
                let approach = Mesh::new_circle(
                    ctx,
                    DrawMode::Stroke(StrokeOptions::default().with_line_width(2.0)),
                    pos,
                    approach_r,
                    1.0,
                    WHITE,
                )?;
                graphics::draw(ctx, &approach, DrawParam::default())?;
            }
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
