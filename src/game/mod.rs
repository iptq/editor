mod background;
mod events;
mod grid;
mod numbers;
mod seeker;
mod sliders;
mod timeline;

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
use imgui::{Window, MenuItem};
use imgui_winit_support::WinitPlatform;
use libosu::{
    beatmap::Beatmap,
    hitobject::{HitObjectKind, SliderSplineKind, SpinnerInfo},
    math::Point,
    spline::Spline,
    timing::{Millis, TimingPoint, TimingPointKind},
};

use crate::audio::{AudioEngine, Sound};
use crate::beatmap::{BeatmapExt, STACK_DISTANCE};
use crate::hitobject::HitObjectExt;
use crate::imgui_wrapper::ImGuiWrapper;
use crate::skin::Skin;
use crate::utils::{self, rect_contains};

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

    frame: usize,
    slider_cache: SliderCache,
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
            frame: 0,
            slider_cache: SliderCache::default(),
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

        struct DrawInfo<'a> {
            hit_object: &'a HitObjectExt,
            fade_opacity: f64,
            end_time: f64,
            color: Color,
            /// Whether or not the circle (slider head for sliders) should appear to have already
            /// been hit in the editor after the object's time has already come.
            circle_is_hit: bool,
        }

        // figure out what objects will be visible on the screen at the current instant
        // 1.5 cus editor so people can see objects for longer durations
        let mut playfield_hitobjects = Vec::new();
        let preempt = 1.5
            * self
                .beatmap
                .inner
                .difficulty
                .approach_preempt()
                .as_seconds();
        let fade_in = 1.5
            * self
                .beatmap
                .inner
                .difficulty
                .approach_fade_time()
                .as_seconds();
        let fade_out_time = fade_in; // TODO: figure out what this number actually is
        let fade_out_offset = preempt; // TODO: figure out what this number actually is

        // TODO: tighten this loop even more by binary searching for the start of the timeline and
        // playfield hitobjects rather than looping through the entire beatmap, better yet, just
        // keeping track of the old index will probably be much faster
        for ho in self.beatmap.hit_objects.iter().rev() {
            let ho_time = ho.inner.start_time.as_seconds();
            let color = self.combo_colors[ho.color_idx];

            // draw in timeline
            self.draw_hitobject_to_timeline(ctx, time, ho)?;

            // draw hitobject in playfield
            let end_time;
            match ho.inner.kind {
                HitObjectKind::Circle => end_time = ho_time,
                HitObjectKind::Slider(_) => {
                    let duration = self.beatmap.inner.get_slider_duration(&ho.inner).unwrap();
                    end_time = ho_time + duration;
                }
                HitObjectKind::Spinner(SpinnerInfo {
                    end_time: spinner_end,
                }) => end_time = spinner_end.as_seconds(),
            };

            let fade_opacity = if time <= ho_time - fade_in {
                // before the hitobject's time arrives, it fades in
                // TODO: calculate ease
                (time - (ho_time - preempt)) / fade_in
            } else if time < ho_time + fade_out_time {
                // while the object is on screen the opacity should be 1
                1.0
            } else if time < end_time + fade_out_offset {
                // after the hitobject's time, it fades out
                // TODO: calculate ease
                ((end_time + fade_out_offset) - time) / fade_out_time
            } else {
                // completely faded out
                0.0
            };
            let circle_is_hit = time > ho_time;

            if ho_time - preempt <= time && time <= end_time + fade_out_offset {
                playfield_hitobjects.push(DrawInfo {
                    hit_object: ho,
                    fade_opacity,
                    end_time,
                    color,
                    circle_is_hit,
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
            let ho_time = ho.inner.start_time.as_seconds();
            let stacking = ho.stacking as f32 * STACK_DISTANCE as f32;
            let pos = [
                PLAYFIELD_BOUNDS.x + osupx_scale_x * (ho.inner.pos.x as f32 - stacking),
                PLAYFIELD_BOUNDS.y + osupx_scale_y * (ho.inner.pos.y as f32 - stacking),
            ];
            let mut color = draw_info.color;
            color.a = 0.6 * draw_info.fade_opacity as f32;

            let mut slider_info = None;
            if let HitObjectKind::Slider(info) = &ho.inner.kind {
                let mut control_points = vec![ho.inner.pos];
                control_points.extend(&info.control_points);

                Game::render_slider_body(
                    &mut self.slider_cache,
                    info,
                    control_points.as_ref(),
                    ctx,
                    PLAYFIELD_BOUNDS,
                    &self.beatmap.inner,
                    color,
                )?;
                slider_info = Some((info, control_points));

                let end_pos = ho.inner.end_pos();
                let end_pos = [
                    PLAYFIELD_BOUNDS.x + osupx_scale_x * end_pos.x as f32,
                    PLAYFIELD_BOUNDS.y + osupx_scale_y * end_pos.y as f32,
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

            // draw main hitcircle
            let faded_color = Color::new(1.0, 1.0, 1.0, 0.6 * draw_info.fade_opacity as f32);
            self.skin.hitcircle.draw(
                ctx,
                (cs_real * 2.0, cs_real * 2.0),
                DrawParam::default().dest(pos).color(color),
            )?;
            self.skin.hitcircleoverlay.draw(
                ctx,
                (cs_real * 2.0, cs_real * 2.0),
                DrawParam::default().dest(pos).color(faded_color),
            )?;

            // draw numbers
            self.draw_numbers_on_circle(ctx, ho.number, pos, cs_real, faded_color)?;

            if let Some((info, control_points)) = slider_info {
                let spline = self.slider_cache.get(&control_points).unwrap();
                Game::render_slider_wireframe(ctx, &control_points, PLAYFIELD_BOUNDS, faded_color)?;

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
                        PLAYFIELD_BOUNDS.x + osupx_scale_x * pos.x as f32,
                        PLAYFIELD_BOUNDS.y + osupx_scale_y * pos.y as f32,
                    ];
                    self.skin.sliderb.draw_frame(
                        ctx,
                        (cs_real * 1.8, cs_real * 1.8),
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

        let mut show = true;
        self.imgui.render(ctx, 1.0, |ui| {
            if let Some(menu_bar) = ui.begin_main_menu_bar() {
                if let Some(menu) = ui.begin_menu("File") {
                    MenuItem::new("Save <C-s>").build(ui);
                    MenuItem::new("Create Difficulty").build(ui);
                    ui.separator();
                    MenuItem::new("Revert to Saved <C-l>").build(ui);
                    MenuItem::new("Export...").build(ui);
                    ui.separator();
                    MenuItem::new("Open Song Folder").build(ui);
                    MenuItem::new("Exit <Esc>").build(ui);
                    menu.end();
                }
                if let Some(menu) = ui.begin_menu("Edit") {
                    menu.end();
                }
                if let Some(menu) = ui.begin_menu("View") {
                    menu.end();
                }
                if let Some(menu) = ui.begin_menu("Compose") {
                    menu.end();
                }
                if let Some(menu) = ui.begin_menu("Design") {
                    menu.end();
                }
                if let Some(menu) = ui.begin_menu("Timing") {
                    menu.end();
                }
                if let Some(menu) = ui.begin_menu("Web") {
                    menu.end();
                }
                if let Some(menu) = ui.begin_menu("Help") {
                    menu.end();
                }
                menu_bar.end();
            }
            // Window
            // Window::new("Hello world")
            //     .size([300.0, 600.0], imgui::Condition::FirstUseEver)
            //     .position([50.0, 50.0], imgui::Condition::FirstUseEver)
            //     .build(&ui, || {
            //         // Your window stuff here!
            //         ui.text("Hi from this label!");
            //     });
            // ui.show_demo_window(&mut show);
        });

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
