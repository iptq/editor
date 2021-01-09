use std::collections::HashMap;
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
    nalgebra::Point2,
    Context, GameError, GameResult,
};
use libosu::{Beatmap, HitObject, HitObjectKind, Point, SpinnerInfo, Spline};

use crate::audio::{AudioEngine, Sound};
use crate::beatmap::{BeatmapExt, STACK_DISTANCE};
use crate::hit_object::HitObjectExt;
use crate::skin::Skin;
use crate::slider_render::render_slider;

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
        // self.beatmap.compute_stacking();

        let dir = path.parent().unwrap();

        let song = Sound::create(dir.join(&self.beatmap.inner.audio_filename))?;
        song.set_position(28.0)?;
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
        const PLAYFIELD_BOUNDS: Rect = Rect::new(112.0, 112.0, 800.0, 600.0);
        const SEEKER_BOUNDS: Rect = Rect::new(46.0, 722.0, 932.0, 36.0);
        const TIMELINE_BOUNDS: Rect = Rect::new(0.0, 0.0, 1024.0, 108.0);

        graphics::clear(ctx, [0.0, 0.0, 0.0, 1.0].into());

        let playfield = Mesh::new_rectangle(
            ctx,
            DrawMode::Stroke(StrokeOptions::default()),
            PLAYFIELD_BOUNDS,
            Color::new(1.0, 1.0, 1.0, 0.5),
        )?;
        graphics::draw(ctx, &playfield, DrawParam::default())?;

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

        let timeline_span = 6.0 / self.beatmap.inner.timeline_zoom;
        let timeline_left = time - timeline_span / 2.0;
        let timeline_right = time + timeline_span / 2.0;
        let timeline_current_line_x = TIMELINE_BOUNDS.x + TIMELINE_BOUNDS.w * 0.5;
        let current_line = Mesh::new_line(
            ctx,
            &[
                Point2::new(timeline_current_line_x, TIMELINE_BOUNDS.y),
                Point2::new(
                    timeline_current_line_x,
                    TIMELINE_BOUNDS.y + TIMELINE_BOUNDS.h,
                ),
            ],
            2.0,
            graphics::WHITE,
        )?;
        graphics::draw(ctx, &current_line, DrawParam::default())?;

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
            if ho_time >= timeline_left && ho_time <= timeline_right {
                let timeline_percent = (ho_time - timeline_left) / (timeline_right - timeline_left);
                let timeline_x = timeline_percent as f32 * TIMELINE_BOUNDS.w + TIMELINE_BOUNDS.x;
                let timeline_y = TIMELINE_BOUNDS.y;
                self.skin.hitcircle.draw(
                    ctx,
                    (TIMELINE_BOUNDS.h, TIMELINE_BOUNDS.h),
                    DrawParam::default()
                        .dest([timeline_x, timeline_y + TIMELINE_BOUNDS.h / 2.0])
                        .offset([0.5, 0.0])
                        .color(color),
                )?;
                self.skin.hitcircleoverlay.draw(
                    ctx,
                    (TIMELINE_BOUNDS.h, TIMELINE_BOUNDS.h),
                    DrawParam::default()
                        .dest([timeline_x, timeline_y + TIMELINE_BOUNDS.h / 2.0])
                        .offset([0.5, 0.0]),
                )?;
            }

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
                PLAYFIELD_BOUNDS.x + osupx_scale_x * ho.inner.pos.0 as f32 - stacking,
                PLAYFIELD_BOUNDS.y + osupx_scale_y * ho.inner.pos.1 as f32 - stacking,
            ];
            let color = draw_info.color;

            let mut slider_info = None;
            if let HitObjectKind::Slider(info) = &ho.inner.kind {
                let color = graphics::Color::new(1.0, 1.0, 1.0, 0.6 * draw_info.opacity as f32);
                let spline = render_slider(
                    &mut self.slider_cache,
                    ctx,
                    PLAYFIELD_BOUNDS,
                    &self.beatmap.inner,
                    &ho.inner,
                    color,
                )?;
                slider_info = Some((info, spline));
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

            if let Some((info, spline)) = slider_info {
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
                    // print!("ho={:.3} ", ho_time);
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

        graphics::present(ctx)?;
        self.frame += 1;
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
