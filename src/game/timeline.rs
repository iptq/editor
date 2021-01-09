use anyhow::Result;
use ggez::{
    graphics::{self, Color, DrawMode, DrawParam, LineCap, Mesh, Rect, StrokeOptions, WHITE},
    nalgebra::Point2,
    Context,
};
use libosu::{hitobject::HitObjectKind, timing::TimingPointKind};

use crate::hitobject::HitObjectExt;

use super::Game;

pub const BOUNDS: Rect = Rect::new(0.0, 54.0, 768.0, 54.0);

pub const RED: Color = Color::new(1.0, 0.0, 0.0, 1.0);
pub const BLUE: Color = Color::new(0.0, 0.0, 1.0, 1.0);
pub const TICKS: &[&[(Color, f32)]] = &[
    &[],
    &[],
    &[],
    &[],
    &[(WHITE, 1.0), (BLUE, 0.5), (RED, 0.5), (BLUE, 0.5)],
];

impl Game {
    pub(super) fn draw_timeline(&self, ctx: &mut Context, time: f64) -> Result<()> {
        let timeline_span = 6.0 / self.beatmap.inner.timeline_zoom;
        let timeline_left = time - timeline_span / 2.0;
        let timeline_right = time + timeline_span / 2.0;
        let timeline_current_line_x = BOUNDS.x + BOUNDS.w * 0.5;

        // the vertical line
        let current_line = Mesh::new_line(
            ctx,
            &[
                Point2::new(timeline_current_line_x, BOUNDS.y),
                Point2::new(timeline_current_line_x, BOUNDS.y + BOUNDS.h),
            ],
            2.0,
            graphics::WHITE,
        )?;
        graphics::draw(ctx, &current_line, DrawParam::default())?;

        // timing sections in this little span
        let mut last_uninherited = None;
        let uninherited_timing_points = self
            .beatmap
            .inner
            .timing_points
            .iter()
            .filter(|x| matches!(x.kind, TimingPointKind::Uninherited(_)))
            .collect::<Vec<_>>();

        for i in 0..uninherited_timing_points.len() {
            let (fst, snd) = (
                &uninherited_timing_points[i],
                uninherited_timing_points.get(i + 1),
            );
            let fst_time = fst.time.as_seconds();
            if let TimingPointKind::Uninherited(info) = &fst.kind {
                last_uninherited = Some(info);
            }

            let snd_time = if let Some(snd) = snd {
                let snd_time = snd.time.as_seconds();
                if snd_time >= timeline_left && snd_time <= timeline_right {
                    Some(snd_time)
                } else {
                    None
                }
            } else {
                None
            };

            if let Some(last_uninherited) = last_uninherited {
                if (fst_time >= timeline_left && fst_time <= timeline_right)
                    || (snd_time.is_some()
                        && snd_time.unwrap() >= timeline_left
                        && snd_time.unwrap() <= timeline_right)
                    || (fst_time < timeline_left
                        && ((snd_time.is_some() && snd_time.unwrap() > timeline_right)
                            || snd_time.is_none()))
                {
                    // TODO: optimize this
                    let beat = last_uninherited.mpb / 1000.0;
                    let ticks = TICKS[last_uninherited.meter as usize];

                    let mut time = fst.time.as_seconds();
                    let passed_measures = ((timeline_left - time) / beat).floor();
                    time += passed_measures * beat;

                    let mut right_limit = timeline_right;
                    if let Some(snd_time) = snd_time {
                        right_limit = right_limit.min(snd_time);
                    }

                    'outer: loop {
                        for i in 0..last_uninherited.meter as usize {
                            let tick_time = time + beat * i as f64 / last_uninherited.meter as f64;
                            if tick_time > right_limit {
                                break 'outer;
                            }

                            let (color, height) = ticks[i];
                            let percent =
                                (tick_time - timeline_left) / (timeline_right - timeline_left);
                            let x = percent as f32 * BOUNDS.w + BOUNDS.x;
                            let y2 = BOUNDS.y + BOUNDS.h;
                            let y1 = y2 - BOUNDS.h * 0.3 * height;
                            let tick = Mesh::new_line(
                                ctx,
                                &[Point2::new(x, y1), Point2::new(x, y2)],
                                1.0,
                                color,
                            )?;
                            graphics::draw(ctx, &tick, DrawParam::default())?;
                        }
                        time += beat;

                        if time >= right_limit {
                            break;
                        }
                    }
                }
            }
        }

        // draw a bottom line for the timeline
        let bottom_line = Mesh::new_line(
            ctx,
            &[
                Point2::new(BOUNDS.x, BOUNDS.y + BOUNDS.h),
                Point2::new(BOUNDS.x + BOUNDS.w, BOUNDS.y + BOUNDS.h),
            ],
            2.0,
            graphics::WHITE,
        )?;
        graphics::draw(ctx, &bottom_line, DrawParam::default())?;

        Ok(())
    }

    pub(super) fn draw_hitobject_to_timeline(
        &self,
        ctx: &mut Context,
        time: f64,
        ho: &HitObjectExt,
    ) -> Result<()> {
        let timeline_span = 6.0 / self.beatmap.inner.timeline_zoom;
        let timeline_left = time - timeline_span / 2.0;
        let timeline_right = time + timeline_span / 2.0;

        let start_time = (ho.inner.start_time.0 as f64) / 1000.0;
        let end_time = self
            .beatmap
            .inner
            .get_hitobject_end_time(&ho.inner)
            .as_seconds();

        let color = self.beatmap.inner.colors[ho.color_idx];
        let color = graphics::Color::new(
            color.red as f32 / 256.0,
            color.green as f32 / 256.0,
            color.blue as f32 / 256.0,
            1.0,
        );

        if end_time >= timeline_left && start_time <= timeline_right {
            let timeline_percent = (start_time - timeline_left) / (timeline_right - timeline_left);
            let head_x = timeline_percent as f32 * BOUNDS.w + BOUNDS.x;
            let timeline_y = BOUNDS.y;

            let tail_percent =
                (end_time.min(timeline_right) - timeline_left) / (timeline_right - timeline_left);
            let tail_x = tail_percent as f32 * BOUNDS.w + BOUNDS.x;

            // draw the slider body on the timeline first
            if let HitObjectKind::Slider(info) = &ho.inner.kind {
                let body_y = BOUNDS.y + BOUNDS.h / 2.0;

                let mut color = color;
                color.a = 0.5;
                let body = Mesh::new_polyline(
                    ctx,
                    DrawMode::Stroke(
                        StrokeOptions::default()
                            .with_line_width(BOUNDS.h)
                            .with_line_cap(LineCap::Round),
                    ),
                    &[Point2::new(head_x, body_y), Point2::new(tail_x, body_y)],
                    color,
                )?;
                graphics::draw(ctx, &body, DrawParam::default())?;

                // draw the slider tail
                if end_time < timeline_right {
                    self.skin.hitcircle.draw(
                        ctx,
                        (BOUNDS.h, BOUNDS.h),
                        DrawParam::default()
                            .dest([tail_x, timeline_y + BOUNDS.h / 2.0])
                            .offset([0.5, 0.0])
                            .color(color),
                    )?;
                    self.skin.hitcircleoverlay.draw(
                        ctx,
                        (BOUNDS.h, BOUNDS.h),
                        DrawParam::default()
                            .dest([tail_x, timeline_y + BOUNDS.h / 2.0])
                            .offset([0.5, 0.0]),
                    )?;
                }

                // draw all visible repeats
                let single_repeat_duration = (end_time - start_time) / info.num_repeats as f64;
                let mut last_visible_repeat = start_time
                    + (((end_time - single_repeat_duration / 2.0).min(timeline_right)
                        - start_time.max(timeline_left))
                        / single_repeat_duration)
                        .floor()
                        * single_repeat_duration;
                while (last_visible_repeat - start_time) > 0.001 {
                    let repeat_percent =
                        (last_visible_repeat - timeline_left) / (timeline_right - timeline_left);
                    let repeat_x = repeat_percent as f32 * BOUNDS.w + BOUNDS.x;
                    self.skin.hitcircle.draw(
                        ctx,
                        (BOUNDS.h, BOUNDS.h),
                        DrawParam::default()
                            .dest([repeat_x, timeline_y + BOUNDS.h / 2.0])
                            .offset([0.5, 0.0])
                            .color(color),
                    )?;
                    self.skin.hitcircleoverlay.draw(
                        ctx,
                        (BOUNDS.h, BOUNDS.h),
                        DrawParam::default()
                            .dest([repeat_x, timeline_y + BOUNDS.h / 2.0])
                            .offset([0.5, 0.0]),
                    )?;
                    self.skin.reversearrow.draw(
                        ctx,
                        (BOUNDS.h / 2.0, BOUNDS.h / 2.0),
                        DrawParam::default()
                            .dest([repeat_x, timeline_y + BOUNDS.h / 2.0])
                            .offset([0.5, 0.5]),
                    )?;
                    last_visible_repeat -= single_repeat_duration;
                }
            }

            // draw the slider head
            self.skin.hitcircle.draw(
                ctx,
                (BOUNDS.h, BOUNDS.h),
                DrawParam::default()
                    .dest([head_x, timeline_y + BOUNDS.h / 2.0])
                    .offset([0.5, 0.0])
                    .color(color),
            )?;
            self.skin.hitcircleoverlay.draw(
                ctx,
                (BOUNDS.h, BOUNDS.h),
                DrawParam::default()
                    .dest([head_x, timeline_y + BOUNDS.h / 2.0])
                    .offset([0.5, 0.0]),
            )?;
        }

        Ok(())
    }
}
