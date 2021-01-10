mod grid;
mod seeker;
mod sliders;
mod timeline;

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use anyhow::Result;
use ggez::{
    event::{EventHandler, KeyCode, KeyMods},
    graphics::{self, Color, DrawParam, FilterMode, Rect, Text, WHITE},
    Context, GameError, GameResult,
};
use libosu::{
    beatmap::Beatmap,
    hitobject::{HitObjectKind, SpinnerInfo},
    math::Point,
    spline::Spline,
    timing::TimingPointKind,
};

use crate::audio::{AudioEngine, Sound};
use crate::beatmap::{BeatmapExt, STACK_DISTANCE};
use crate::hitobject::HitObjectExt;
use crate::skin::Skin;

pub const PLAYFIELD_BOUNDS: Rect = Rect::new(112.0, 122.0, 800.0, 600.0);

pub type SliderCache = HashMap<Vec<Point<i32>>, Spline>;

pub struct Game {
    is_playing: bool,
    audio_engine: AudioEngine,
    song: Option<Sound>,
    beatmap: BeatmapExt,
    pub skin: Skin,
    frame: usize,
    slider_cache: SliderCache,
}

impl Game {
    pub fn new() -> Result<Game> {
        let audio_engine = AudioEngine::new()?;
        let skin = Skin::new();

        let beatmap = Beatmap::default();
        let beatmap = BeatmapExt::new(beatmap);

        Ok(Game {
            is_playing: false,
            audio_engine,
            beatmap,
            song: None,
            skin,
            frame: 0,
            slider_cache: SliderCache::default(),
        })
    }

    pub fn load_beatmap(&mut self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();

        let mut file = File::open(&path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let beatmap = Beatmap::from_osz(&contents)?;
        self.beatmap = BeatmapExt::new(beatmap);
        self.beatmap.compute_colors();
        self.beatmap.compute_stacking();

        let dir = path.parent().unwrap();

        let song = Sound::create(dir.join(&self.beatmap.inner.audio_filename))?;
        self.song = Some(song);

        Ok(())
    }

    pub fn jump_to_time(&mut self, time: f64) -> Result<()> {
        if let Some(song) = &self.song {
            song.set_position(time)?;
        }
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

    fn draw_helper(&mut self, ctx: &mut Context) -> Result<()> {
        // TODO: lol

        graphics::clear(ctx, [0.0, 0.0, 0.0, 1.0].into());

        self.draw_grid(ctx)?;

        let time = self.song.as_ref().unwrap().position()?;
        let text = Text::new(format!("time: {}", time).as_ref());
        graphics::queue_text(ctx, &text, [0.0, 0.0], Some(WHITE));
        graphics::draw_queued_text(ctx, DrawParam::default(), None, FilterMode::Linear)?;

        struct DrawInfo<'a> {
            hit_object: &'a HitObjectExt,
            opacity: f64,
            end_time: f64,
            color: Color,
        }

        let mut playfield_hitobjects = Vec::new();
        let preempt = (self.beatmap.inner.difficulty.approach_preempt() as f64) / 1000.0;
        let fade_in = (self.beatmap.inner.difficulty.approach_fade_time() as f64) / 1000.0;

        // TODO: tighten this loop even more by binary searching for the start of the timeline and
        // playfield hitobjects rather than looping through the entire beatmap, better yet, just
        // keeping track of the old index will probably be much faster
        for ho in self.beatmap.hit_objects.iter().rev() {
            let ho_time = (ho.inner.start_time.0 as f64) / 1000.0;
            let color = self.beatmap.inner.colors[ho.color_idx];
            let color = graphics::Color::new(
                color.red as f32 / 256.0,
                color.green as f32 / 256.0,
                color.blue as f32 / 256.0,
                1.0,
            );

            // draw in timeline
            self.draw_hitobject_to_timeline(ctx, time, ho)?;

            // draw hitobject in playfield
            let end_time;
            let opacity = if time > ho_time - fade_in {
                1.0
            } else {
                // TODO: calculate ease
                (time - (ho_time - preempt)) / fade_in
            };
            match ho.inner.kind {
                HitObjectKind::Circle => end_time = ho_time,
                HitObjectKind::Slider(_) => {
                    let duration = self.beatmap.inner.get_slider_duration(&ho.inner).unwrap();
                    end_time = ho_time + duration / 1000.0;
                }
                HitObjectKind::Spinner(SpinnerInfo {
                    end_time: spinner_end,
                }) => end_time = (spinner_end.0 as f64) / 1000.0,
            };
            if ho_time - preempt < time && time < end_time {
                playfield_hitobjects.push(DrawInfo {
                    hit_object: ho,
                    opacity,
                    end_time,
                    color,
                });
            }
        }

        self.draw_timeline(ctx, time)?;

        let cs_scale = PLAYFIELD_BOUNDS.w / 640.0;
        let osupx_scale_x = PLAYFIELD_BOUNDS.w / 512.0;
        let osupx_scale_y = PLAYFIELD_BOUNDS.h / 384.0;
        let cs_osupx = self.beatmap.inner.difficulty.circle_size_osupx();
        let cs_real = cs_osupx * cs_scale;

        for draw_info in playfield_hitobjects.iter() {
            let ho = draw_info.hit_object;
            let ho_time = (ho.inner.start_time.0 as f64) / 1000.0;
            let stacking = ho.stacking as f32 * STACK_DISTANCE as f32;
            let pos = [
                PLAYFIELD_BOUNDS.x + osupx_scale_x * (ho.inner.pos.0 as f32 - stacking),
                PLAYFIELD_BOUNDS.y + osupx_scale_y * (ho.inner.pos.1 as f32 - stacking),
            ];
            let color = draw_info.color;

            let mut slider_info = None;
            if let HitObjectKind::Slider(info) = &ho.inner.kind {
                let mut control_points = vec![ho.inner.pos];
                control_points.extend(&info.control_points);

                let mut color = color.clone();
                color.a = 0.6 * draw_info.opacity as f32;
                let spline = Game::render_slider(
                    &mut self.slider_cache,
                    info,
                    control_points.as_ref(),
                    ctx,
                    PLAYFIELD_BOUNDS,
                    &self.beatmap.inner,
                    &ho.inner,
                    color,
                )?;
                slider_info = Some((info, control_points, spline));

                let end_pos = ho.inner.end_pos().unwrap();
                let end_pos = [
                    PLAYFIELD_BOUNDS.x + osupx_scale_x * end_pos.0 as f32,
                    PLAYFIELD_BOUNDS.y + osupx_scale_y * end_pos.1 as f32,
                ];
                self.skin.hitcircle.draw(
                    ctx,
                    (cs_real * 2.0, cs_real * 2.0),
                    DrawParam::default().dest(end_pos).color(color),
                )?;
                self.skin.hitcircleoverlay.draw(
                    ctx,
                    (cs_real * 2.0, cs_real * 2.0),
                    DrawParam::default().dest(end_pos),
                )?;
            }

            self.skin.hitcircle.draw(
                ctx,
                (cs_real * 2.0, cs_real * 2.0),
                DrawParam::default().dest(pos).color(color),
            )?;
            self.skin.hitcircleoverlay.draw(
                ctx,
                (cs_real * 2.0, cs_real * 2.0),
                DrawParam::default().dest(pos),
            )?;

            if let Some((info, control_points, spline)) = slider_info {
                Game::render_slider_wireframe(ctx, &control_points, PLAYFIELD_BOUNDS)?;

                if time > ho_time && time < draw_info.end_time {
                    let elapsed_time = time - ho_time;
                    let total_duration = draw_info.end_time - ho_time;
                    let single_duration = total_duration / info.num_repeats as f64;
                    let finished_repeats = (elapsed_time / single_duration).floor();
                    let this_repeat_time = elapsed_time - finished_repeats * single_duration;
                    let mut travel_percent = this_repeat_time / single_duration;
                    // reverse direction on odd trips
                    if finished_repeats as u32 % 2 == 1 {
                        travel_percent = 1.0 - travel_percent;
                    }
                    let travel_length = travel_percent * info.pixel_length;
                    let pos = spline.point_at_length(travel_length);
                    let ball_pos = [
                        PLAYFIELD_BOUNDS.x + osupx_scale_x * pos.0 as f32,
                        PLAYFIELD_BOUNDS.y + osupx_scale_y * pos.1 as f32,
                    ];
                    self.skin.sliderb.draw_frame(
                        ctx,
                        (cs_real * 2.0, cs_real * 2.0),
                        DrawParam::default().dest(ball_pos).color(color),
                        (travel_percent / 0.25) as usize,
                    )?;
                }
            }

            if time < ho_time {
                let time_diff = ho_time - time;
                let approach_r = cs_real * (1.0 + 2.0 * time_diff as f32 / 0.75);
                self.skin.approachcircle.draw(
                    ctx,
                    (approach_r * 2.0, approach_r * 2.0),
                    DrawParam::default().dest(pos).color(color),
                )?;
            }
        }

        self.draw_seeker(ctx)?;

        graphics::present(ctx)?;
        self.frame += 1;
        Ok(())
    }

    fn seek_by_steps(&self, n: i32) -> Result<()> {
        if let Some(song) = &self.song {
            let pos = song.position()?;
            let mut delta = None;
            for timing_point in self.beatmap.inner.timing_points.iter() {
                if let TimingPointKind::Uninherited(info) = &timing_point.kind {
                    if pos > timing_point.time.as_seconds() {
                        let diff = pos - timing_point.time.as_seconds();
                        let tick = info.mpb / 1000.0 / info.meter as f64;
                        let beats = (diff / tick).round();
                        let frac = diff - beats * tick;
                        if frac.abs() < 0.0001 {
                            delta = Some(n as f64 * tick);
                        } else {
                            if n > 0 {
                                delta = Some((n - 1) as f64 * tick + (tick - frac));
                            } else {
                                delta = Some((n - 1) as f64 * tick - frac);
                            }
                        }
                        break;
                    }
                }
            }
            if let Some(delta) = delta {
                song.set_position(pos + delta)?;
            }
        }
        Ok(())
    }
}

impl EventHandler for Game {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn key_up_event(&mut self, _: &mut Context, keycode: KeyCode, _: KeyMods) {
        use KeyCode::*;
        match keycode {
            Space => self.toggle_playing(),
            Colon => {}
            _ => {}
        };
    }

    fn key_down_event(&mut self, _: &mut Context, keycode: KeyCode, _: KeyMods, _: bool) {
        use KeyCode::*;
        match keycode {
            Left => {
                self.seek_by_steps(-1);
            }
            Right => {
                self.seek_by_steps(1);
            }
            _ => {}
        };
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        if let Err(err) = self.draw_helper(ctx) {
            return Err(GameError::RenderError(err.to_string()));
        };
        Ok(())
    }
}
