mod background;
mod events;
mod grid;
mod hitobjects;
mod numbers;
mod seeker;
mod sliders;
mod timeline;
mod ui;

use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::str::FromStr;

use anyhow::Result;
use ggez::{
    event::{KeyCode, MouseButton},
    graphics::{
        self, CanvasGeneric, Color, DrawMode, DrawParam, FilterMode, GlBackendSpec, Image, Mesh,
        Rect, StrokeOptions, Text,
    },
    Context,
};
use image::io::Reader as ImageReader;
use libosu::{
    beatmap::Beatmap,
    hitobject::{HitObjectKind, SliderSplineKind},
    math::Point,
    spline::Spline,
    timing::{Millis, TimingPoint, TimingPointKind},
};

use crate::audio::{AudioEngine, Sound};
use crate::beatmap::BeatmapExt;
use crate::hitobject::HitObjectExt;
use crate::imgui_wrapper::ImGuiWrapper;
use crate::skin::Skin;
use crate::utils::{self, rect_contains};

use self::ui::UiState;

pub const PLAYFIELD_BOUNDS: Rect = Rect::new(112.0, 122.0, 800.0, 600.0);
pub const DEFAULT_COLORS: &[(f32, f32, f32)] = &[
    (1.0, 0.75, 0.0),
    (0.0, 0.8, 0.0),
    (0.07, 0.5, 1.0),
    (0.95, 0.1, 0.22),
];

pub type SliderCache = HashMap<Vec<Point<i32>>, Spline>;

pub struct PartialSliderState {
    start_time: Millis,
    kind: SliderSplineKind,
    control_points: Vec<Point<i32>>,
    pixel_length: f64,
}

#[derive(Clone, Debug)]
pub enum Tool {
    Select,
    Circle,
    Slider,
}

pub struct Game {
    is_playing: bool,
    imgui: ImGuiWrapper,
    audio_engine: AudioEngine,
    song: Option<Sound>,
    beatmap: BeatmapExt,
    pub skin: Skin,
    background_image: Option<Image>,
    ui_state: Option<UiState>,

    frame: usize,
    slider_cache: SliderCache,
    seeker_drag: bool,
    seeker_cache: Option<CanvasGeneric<GlBackendSpec>>,
    combo_colors: Vec<Color>,
    selected_objects: Vec<usize>,
    tool: Tool,
    partial_slider_state: Option<PartialSliderState>,

    keymap: HashSet<KeyCode>,
    mouse_pos: (f32, f32),
    left_drag_start: Option<(f32, f32)>,
    right_drag_start: Option<(f32, f32)>,
    current_uninherited_timing_point: Option<TimingPoint>,
    current_inherited_timing_point: Option<TimingPoint>,
}

impl Game {
    pub fn new(imgui: ImGuiWrapper) -> Result<Game> {
        let audio_engine = AudioEngine::new()?;
        let skin = Skin::new();

        let beatmap = Beatmap::default();
        let beatmap = BeatmapExt::new(beatmap);

        Ok(Game {
            is_playing: false,
            imgui,
            audio_engine,
            beatmap,
            song: None,
            skin,
            ui_state: Some(UiState::default()),
            frame: 0,
            slider_cache: SliderCache::default(),
            seeker_drag: false,
            seeker_cache: None,
            combo_colors: DEFAULT_COLORS
                .iter()
                .map(|(r, g, b)| Color::new(*r, *g, *b, 1.0))
                .collect(),
            background_image: None,
            selected_objects: vec![],
            keymap: HashSet::new(),
            mouse_pos: (-1.0, -1.0),
            left_drag_start: None,
            right_drag_start: None,
            tool: Tool::Select,
            partial_slider_state: None,
            current_uninherited_timing_point: None,
            current_inherited_timing_point: None,
        })
    }

    pub fn load_beatmap(&mut self, ctx: &mut Context, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();

        let mut file = File::open(&path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let beatmap = Beatmap::from_str(&contents)?;
        self.beatmap = BeatmapExt::new(beatmap);
        self.beatmap.compute_stacking();

        if !self.beatmap.inner.colors.is_empty() {
            self.combo_colors.clear();
            self.combo_colors = self
                .beatmap
                .inner
                .colors
                .iter()
                .map(|color| {
                    Color::new(
                        color.red as f32 / 255.0,
                        color.green as f32 / 255.0,
                        color.blue as f32 / 255.0,
                        1.0,
                    )
                })
                .collect();
        } else {
            self.combo_colors = DEFAULT_COLORS
                .iter()
                .map(|(r, g, b)| Color::new(*r, *g, *b, 1.0))
                .collect();
        }
        self.beatmap.compute_colors(&self.combo_colors);

        let dir = path.parent().unwrap();

        // TODO: more background images possible?
        for evt in self.beatmap.inner.events.iter() {
            use libosu::events::Event;
            if let Event::Background(evt) = evt {
                let path = utils::fuck_you_windows(dir, &evt.filename)?;
                if let Some(path) = path {
                    let img = ImageReader::open(path)?.decode()?;
                    let img_buf = img.into_rgba8();
                    let image = Image::from_rgba8(
                        ctx,
                        img_buf.width() as u16,
                        img_buf.height() as u16,
                        img_buf.as_raw(),
                    )?;
                    self.background_image = Some(image);
                }
            }
        }

        let song = Sound::create(dir.join(&self.beatmap.inner.audio_filename))?;
        song.set_volume(0.1);
        self.song = Some(song);
        self.timestamp_changed()?;

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

        self.draw_background(ctx)?;
        self.draw_grid(ctx)?;

        let time = self.song.as_ref().unwrap().position()?;
        let time_millis = Millis::from_seconds(time);
        let text = Text::new(
            format!(
                "tool: {:?} time: {:.4}, mouse: {:?}",
                self.tool, time, self.mouse_pos
            )
            .as_ref(),
        );
        graphics::queue_text(ctx, &text, [0.0, 0.0], Some(Color::WHITE));
        graphics::draw_queued_text(ctx, DrawParam::default(), None, FilterMode::Linear)?;

        self.draw_timeline(ctx, time)?;

        self.draw_hitobjects(ctx, time)?;

        self.draw_seeker(ctx)?;

        // TODO: don't duplicate these from hitobjects.rs
        let cs_scale = PLAYFIELD_BOUNDS.w / 640.0;
        let cs_osupx = self.beatmap.inner.difficulty.circle_size_osupx();
        let cs_real = cs_osupx * cs_scale;

        // draw whatever tool user is using
        let (mx, my) = self.mouse_pos;
        let pos_x = (mx - PLAYFIELD_BOUNDS.x) / PLAYFIELD_BOUNDS.w * 512.0;
        let pos_y = (my - PLAYFIELD_BOUNDS.y) / PLAYFIELD_BOUNDS.h * 384.0;
        let mouse_pos = Point::new(pos_x as i32, pos_y as i32);
        match self.tool {
            Tool::Select => {
                let (mx, my) = self.mouse_pos;
                if let Some((dx, dy)) = self.left_drag_start {
                    if rect_contains(&PLAYFIELD_BOUNDS, dx, dy) {
                        let ax = dx.min(mx);
                        let ay = dy.min(my);
                        let bx = dx.max(mx);
                        let by = dy.max(my);
                        let drag_rect = Rect::new(ax, ay, bx - ax, by - ay);
                        let drag_rect = Mesh::new_rectangle(
                            ctx,
                            DrawMode::Stroke(StrokeOptions::default()),
                            drag_rect,
                            Color::WHITE,
                        )?;
                        graphics::draw(ctx, &drag_rect, DrawParam::default())?;
                    }
                }
            }
            Tool::Circle => {
                if rect_contains(&PLAYFIELD_BOUNDS, mx, my) {
                    let pos = [mx, my];
                    let color = Color::new(1.0, 1.0, 1.0, 0.4);
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
                }
            }
            Tool::Slider => {
                let color = Color::new(1.0, 1.0, 1.0, 0.4);
                if let Some(state) = &mut self.partial_slider_state {
                    let mut nodes = state.control_points.clone();
                    let mut kind = state.kind;
                    if let Some(last) = nodes.last() {
                        if mouse_pos != *last {
                            nodes.push(mouse_pos);
                            kind = upgrade_slider_type(kind, nodes.len());
                        }
                    }

                    if nodes.len() > 1 && !(nodes.len() == 2 && nodes[0] == nodes[1]) {
                        let slider_velocity =
                            self.beatmap.inner.get_slider_velocity_at_time(time_millis);
                        let slider_multiplier = self.beatmap.inner.difficulty.slider_multiplier;
                        let pixels_per_beat = slider_multiplier * 100.0 * slider_velocity;
                        let pixels_per_tick = pixels_per_beat / 4.0; // TODO: FIX!!!

                        let mut spline = Spline::from_control(kind, &nodes, None);
                        let len = spline.pixel_length();
                        debug!("original len: {}", len);
                        let num_ticks = (len / pixels_per_tick).floor();
                        debug!("num ticks: {}", num_ticks);

                        let fixed_len = num_ticks * pixels_per_tick;
                        state.pixel_length = fixed_len;
                        debug!("fixed len: {}", fixed_len);
                        spline.truncate(fixed_len);

                        debug!("len: {}", spline.pixel_length());
                        Game::render_spline(
                            ctx,
                            &self.beatmap.inner,
                            &spline,
                            PLAYFIELD_BOUNDS,
                            color,
                        )?;
                        debug!("done rendering slider body");
                    }

                    Game::render_slider_wireframe(ctx, &nodes, PLAYFIELD_BOUNDS, Color::WHITE)?;
                    debug!("done rendering slider wireframe");
                } else {
                    if rect_contains(&PLAYFIELD_BOUNDS, mx, my) {
                        let pos = [mx, my];
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
                    }
                }
            }
            _ => {}
        }

        if let Some(mut state) = self.ui_state.take() {
            self.draw_ui(ctx, &mut state)?;
            self.ui_state = Some(state);
        }

        graphics::present(ctx)?;
        self.frame += 1;
        if self.is_playing {
            self.timestamp_changed()?;
        }

        Ok(())
    }

    fn timestamp_changed(&mut self) -> Result<()> {
        if let Some(song) = &self.song {
            let pos = song.position()?;

            if let Some(timing_point) = self.beatmap.inner.timing_points.first() {
                if pos < timing_point.time.as_seconds() {
                    if let TimingPointKind::Uninherited(_) = &timing_point.kind {
                        self.current_uninherited_timing_point = Some(timing_point.clone());
                    }
                }
            }

            let mut found_uninherited = false;
            let mut found_inherited = false;
            for timing_point in self.beatmap.inner.timing_points.iter() {
                if pos < timing_point.time.as_seconds() {
                    continue;
                }

                match &timing_point.kind {
                    TimingPointKind::Uninherited(_) => {
                        self.current_uninherited_timing_point = Some(timing_point.clone());
                        found_uninherited = true;
                    }
                    TimingPointKind::Inherited(_) => {
                        self.current_inherited_timing_point = Some(timing_point.clone());
                        found_inherited = true;
                    }
                }

                if found_inherited && found_uninherited {
                    break;
                }
            }
        }

        Ok(())
    }

    fn seek_by_steps(&mut self, n: i32) -> Result<()> {
        if let Some(song) = &self.song {
            let pos = song.position()?;
            let mut delta = None;
            if let Some(TimingPoint {
                kind: TimingPointKind::Uninherited(info),
                time,
                ..
            }) = &self.current_uninherited_timing_point
            {
                let diff = pos - time.as_seconds();
                let tick = info.mpb / 1000.0 / info.meter as f64;
                let beats = (diff / tick).round();
                let frac = diff - beats * tick;
                if frac.abs() < 0.0001 {
                    delta = Some(n as f64 * tick);
                } else if n > 0 {
                    delta = Some((n - 1) as f64 * tick + (tick - frac));
                } else {
                    delta = Some((n - 1) as f64 * tick - frac);
                }
            }
            if let Some(delta) = delta {
                song.set_position(pos + delta)?;
                self.timestamp_changed()?;
            }
        }
        Ok(())
    }

    fn handle_click(&mut self, btn: MouseButton, x: f32, y: f32) -> Result<()> {
        println!("handled click {}", self.song.is_some());
        let pos_x = (x - PLAYFIELD_BOUNDS.x) / PLAYFIELD_BOUNDS.w * 512.0;
        let pos_y = (y - PLAYFIELD_BOUNDS.y) / PLAYFIELD_BOUNDS.h * 384.0;
        let pos = Point::new(pos_x as i32, pos_y as i32);

        if let Some(song) = &self.song {
            println!("song exists! {:?} {:?}", btn, self.tool);
            let time = song.position()?;
            let time_millis = Millis::from_seconds(time);

            if let (MouseButton::Left, Tool::Select) = (btn, &self.tool) {
            } else if let (MouseButton::Left, Tool::Circle) = (btn, &self.tool) {
                println!("left, circle, {:?} {} {}", PLAYFIELD_BOUNDS, x, y);
                if rect_contains(&PLAYFIELD_BOUNDS, x, y) {
                    let time = Millis::from_seconds(song.position()?);
                    match self
                        .beatmap
                        .hit_objects
                        .binary_search_by_key(&time, |ho| ho.inner.start_time)
                    {
                        Ok(v) => {
                            println!("unfortunately already found at idx {}", v);
                        }
                        Err(idx) => {
                            use libosu::{
                                hitobject::HitObject,
                                hitsounds::{Additions, SampleInfo},
                            };

                            let inner = HitObject {
                                start_time: time,
                                pos,
                                kind: HitObjectKind::Circle,
                                new_combo: false,
                                skip_color: 0,
                                additions: Additions::empty(),
                                sample_info: SampleInfo::default(),
                            };

                            let new_obj = HitObjectExt {
                                inner,
                                stacking: 0,
                                color_idx: 0,
                                number: 0,
                            };
                            println!("creating new hitobject: {:?}", new_obj);
                            self.beatmap.hit_objects.insert(idx, new_obj);
                            self.beatmap.compute_stacking();
                            self.beatmap.compute_colors(&self.combo_colors);
                        }
                    }
                }
            } else if let (MouseButton::Left, Tool::Slider) = (btn, &self.tool) {
                if let Some(PartialSliderState {
                    kind,
                    control_points: ref mut nodes,
                    ..
                }) = &mut self.partial_slider_state
                {
                    nodes.push(pos);
                    *kind = upgrade_slider_type(*kind, nodes.len());
                } else {
                    self.partial_slider_state = Some(PartialSliderState {
                        start_time: time_millis,
                        kind: SliderSplineKind::Linear,
                        control_points: vec![pos],
                        pixel_length: 0.0,
                    });
                }
            } else if let (MouseButton::Right, Tool::Slider) = (btn, &self.tool) {
                if let Some(state) = &mut self.partial_slider_state {
                    match self
                        .beatmap
                        .hit_objects
                        .binary_search_by_key(&state.start_time.0, |ho| ho.inner.start_time.0)
                    {
                        Ok(v) => {
                            println!("unfortunately already found at idx {}", v);
                        }
                        Err(idx) => {
                            use libosu::{
                                hitobject::{HitObject, SliderInfo},
                                hitsounds::{Additions, SampleInfo},
                            };

                            state.control_points.push(pos);
                            let after_len = state.control_points.len();
                            let first = state.control_points.remove(0);
                            let inner = HitObject {
                                start_time: state.start_time,
                                pos: first,
                                kind: HitObjectKind::Slider(SliderInfo {
                                    kind: upgrade_slider_type(state.kind, after_len),
                                    control_points: state.control_points.clone(),
                                    num_repeats: 1,
                                    pixel_length: state.pixel_length,
                                    edge_additions: vec![],
                                    edge_samplesets: vec![],
                                }),
                                new_combo: false,
                                skip_color: 0,
                                additions: Additions::empty(),
                                sample_info: SampleInfo::default(),
                            };

                            let new_obj = HitObjectExt {
                                inner,
                                stacking: 0,
                                color_idx: 0,
                                number: 0,
                            };
                            println!("creating new hitobject: {:?}", new_obj);
                            self.beatmap.hit_objects.insert(idx, new_obj);
                            self.beatmap.compute_stacking();
                            self.beatmap.compute_colors(&self.combo_colors);
                        }
                    }
                    self.partial_slider_state = None;
                }
            }
        }
        Ok(())
    }

    fn switch_tool_to(&mut self, target: Tool) {
        // clear slider state if we're switching away from slider
        if matches!(self.tool, Tool::Slider) && !matches!(target, Tool::Slider) {
            self.partial_slider_state = None;
        }

        self.tool = target;
    }
}

fn upgrade_slider_type(initial_type: SliderSplineKind, after_len: usize) -> SliderSplineKind {
    match (initial_type, after_len) {
        (SliderSplineKind::Linear, 3) => SliderSplineKind::Perfect,
        (SliderSplineKind::Perfect, 4) => SliderSplineKind::Bezier,
        _ => initial_type,
    }
}
