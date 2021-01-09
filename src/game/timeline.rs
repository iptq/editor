use anyhow::Result;
use ggez::{
    graphics::{self, DrawParam, Mesh, Rect},
    nalgebra::Point2,
    Context,
};

use crate::hit_object::HitObjectExt;

use super::Game;

pub const TIMELINE_BOUNDS: Rect = Rect::new(0.0, 0.0, 1024.0, 108.0);

impl Game {
    pub(super) fn draw_timeline(&self, ctx: &mut Context, time: f64) -> Result<()> {
        let timeline_span = 6.0 / self.beatmap.inner.timeline_zoom;
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
        let color = self.beatmap.inner.colors[ho.color_idx];
        let color = graphics::Color::new(
            color.red as f32 / 256.0,
            color.green as f32 / 256.0,
            color.blue as f32 / 256.0,
            1.0,
        );

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

        Ok(())
    }
}
