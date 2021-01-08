use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use anyhow::Result;
use ggez::{
    event::{EventHandler, KeyCode, KeyMods},
    nalgebra::Point2,
    graphics::{
        self, Color, DrawMode, DrawParam, FillOptions, FilterMode, Mesh, Rect, StrokeOptions, Text,
        WHITE,
    },
    Context, GameError, GameResult,
};
use libosu::{Beatmap, HitObject, HitObjectKind, Point, SpinnerInfo};

use crate::audio::{AudioEngine, Sound};
use crate::skin::Skin;
use crate::slider_render::{render_slider, Spline};

pub type SliderCache = HashMap<Vec<Point<i32>>, Spline>;

pub struct Game {
    is_playing: bool,
    audio_engine: AudioEngine,
    song: Option<Sound>,
    beatmap: Beatmap,
    hit_objects: Vec<HitObject>,
    pub skin: Skin,
    frame: usize,
    slider_cache: SliderCache,
}

impl Game {
    pub fn new() -> Result<Game> {
        let audio_engine = AudioEngine::new()?;
        let beatmap = Beatmap::default();
        let hit_objects = Vec::new();
        let skin = Skin::new();

        Ok(Game {
            is_playing: false,
            audio_engine,
            beatmap,
            hit_objects,
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
            hit_object: &'a HitObject,
            opacity: f64,
            end_time: f64,
        }

        let timeline_span = 6.0 / self.beatmap.timeline_zoom;
        let timeline_left = time - timeline_span / 2.0;
        let timeline_right = time + timeline_span / 2.0;
        println!("left {:.3} right {:.3}", timeline_left, timeline_right);
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
        let preempt = (self.beatmap.difficulty.approach_preempt() as f64) / 1000.0;
        let fade_in = (self.beatmap.difficulty.approach_fade_time() as f64) / 1000.0;

        // TODO: tighten this loop even more by binary searching for the start of the timeline and
        // playfield hitobjects rather than looping through the entire beatmap, better yet, just
        // keeping track of the old index will probably be much faster
        for ho in self.beatmap.hit_objects.iter().rev() {
            let ho_time = (ho.start_time.0 as f64) / 1000.0;

            // draw in timeline
            if ho_time >= timeline_left && ho_time <= timeline_right {
                let timeline_percent = (ho_time - timeline_left) / (timeline_right - timeline_left);
                let timeline_x = timeline_percent as f32 * TIMELINE_BOUNDS.w + TIMELINE_BOUNDS.x;
                let timeline_y = TIMELINE_BOUNDS.y;
                println!(
                    " - [{}] {:.3}-{:.3} : {:.3}%",
                    self.beatmap.timeline_zoom,
                    timeline_left,
                    timeline_right,
                    timeline_percent * 100.0
                );
                self.skin.hitcircle.draw(
                    ctx,
                    (TIMELINE_BOUNDS.h, TIMELINE_BOUNDS.h),
                    DrawParam::default()
                        .dest([timeline_x, timeline_y + TIMELINE_BOUNDS.h / 2.0])
                        .offset([0.5, 0.0]),
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
                playfield_hitobjects.push(DrawInfo {
                    hit_object: ho,
                    opacity,
                    end_time,
                });
            }
        }

        let cs_scale = PLAYFIELD_BOUNDS.w / 640.0;
        let osupx_scale_x = PLAYFIELD_BOUNDS.w / 512.0;
        let osupx_scale_y = PLAYFIELD_BOUNDS.h / 384.0;
        let cs_osupx = self.beatmap.difficulty.circle_size_osupx();
        let cs_real = cs_osupx * cs_scale;

        for draw_info in playfield_hitobjects.iter() {
            let ho = draw_info.hit_object;
            let ho_time = (ho.start_time.0 as f64) / 1000.0;
            let pos = [
                PLAYFIELD_BOUNDS.x + osupx_scale_x * ho.pos.0 as f32,
                PLAYFIELD_BOUNDS.y + osupx_scale_y * ho.pos.1 as f32,
            ];
            let color = graphics::Color::new(1.0, 1.0, 1.0, draw_info.opacity as f32);

            if let HitObjectKind::Slider(info) = &ho.kind {
                let color = graphics::Color::new(1.0, 1.0, 1.0, 0.6 * draw_info.opacity as f32);
                let spline = render_slider(
                    &mut self.slider_cache,
                    ctx,
                    PLAYFIELD_BOUNDS,
                    &self.beatmap,
                    ho,
                    color,
                )?;

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
                        DrawParam::default().dest(ball_pos),
                        (travel_percent / 0.25) as usize,
                    )?;
                }
            }

            self.skin.hitcircle.draw(
                ctx,
                (cs_real * 2.0, cs_real * 2.0),
                DrawParam::default().dest(pos).color(color),
            )?;

            self.skin.hitcircleoverlay.draw(
                ctx,
                (cs_real * 2.0, cs_real * 2.0),
                DrawParam::default().dest(pos).color(color),
            )?;

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
