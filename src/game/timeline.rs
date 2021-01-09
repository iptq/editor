use anyhow::Result;
use ggez::{
    graphics::{self, Color, DrawParam, Mesh, Rect, WHITE},
    nalgebra::Point2,
    Context,
};
use libosu::TimingPointKind;

use crate::hit_object::HitObjectExt;

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
        for timing_points in self.beatmap.inner.timing_points.windows(2) {
            let (fst, snd) = (&timing_points[0], &timing_points[1]);
            let fst_time = fst.time.as_seconds();
            let snd_time = snd.time.as_seconds();
            if let TimingPointKind::Uninherited(info) = &fst.kind {
                last_uninherited = Some(info);
            }

            if let Some(last_uninherited) = last_uninherited {
                if (fst_time >= timeline_left && fst_time <= timeline_right)
                    || (snd_time >= timeline_left && snd_time <= timeline_right)
                    || (fst_time < timeline_left && snd_time > timeline_right)
                {
                    // TODO: optimize this
                    let mut time = fst.time.as_seconds();
                    let beat = last_uninherited.mpb / 1000.0;
                    let ticks = TICKS[last_uninherited.meter as usize];
                    'outer: loop {
                        for i in 0..last_uninherited.meter as usize {
                            let tick_time = time + beat * i as f64 / last_uninherited.meter as f64;
                            if tick_time > snd_time.min(timeline_right) {
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

                        if time >= snd_time.min(timeline_right) {
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

        let ho_time = (ho.inner.start_time.0 as f64) / 1000.0;
        let end_time = (self.beatmap.inner.get_hitobject_end_time(&ho.inner).0 as f64) / 1000.0;

        let color = self.beatmap.inner.colors[ho.color_idx];
        let color = graphics::Color::new(
            color.red as f32 / 256.0,
            color.green as f32 / 256.0,
            color.blue as f32 / 256.0,
            1.0,
        );

        if end_time >= timeline_left && ho_time <= timeline_right {
            let timeline_percent = (ho_time - timeline_left) / (timeline_right - timeline_left);
            let timeline_x = timeline_percent as f32 * BOUNDS.w + BOUNDS.x;
            let timeline_y = BOUNDS.y;
            self.skin.hitcircle.draw(
                ctx,
                (BOUNDS.h, BOUNDS.h),
                DrawParam::default()
                    .dest([timeline_x, timeline_y + BOUNDS.h / 2.0])
                    .offset([0.5, 0.0])
                    .color(color),
            )?;
            self.skin.hitcircleoverlay.draw(
                ctx,
                (BOUNDS.h, BOUNDS.h),
                DrawParam::default()
                    .dest([timeline_x, timeline_y + BOUNDS.h / 2.0])
                    .offset([0.5, 0.0]),
            )?;
        }

        Ok(())
    }
}
