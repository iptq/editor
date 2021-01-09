use anyhow::Result;
use ggez::{
    graphics::{self, DrawParam, Mesh, Rect},
    nalgebra::Point2,
    Context,
};

use super::Game;

pub const BOUNDS: Rect = Rect::new(46.0, 722.0, 932.0, 36.0);

impl Game {
    pub(super) fn draw_seeker(&self, ctx: &mut Context) -> Result<()> {
        let line_y = BOUNDS.y + BOUNDS.h / 2.0;
        let line = Mesh::new_line(
            ctx,
            &[
                Point2::new(BOUNDS.x, line_y),
                Point2::new(BOUNDS.x + BOUNDS.w, line_y),
            ],
            1.0,
            graphics::WHITE,
        )?;
        graphics::draw(ctx, &line, DrawParam::default())?;

        if let Some(song) = &self.song {
            let percent = song.position()? / song.length()?;
            let x = BOUNDS.x + percent as f32 * BOUNDS.w;
            let line = Mesh::new_line(
                ctx,
                &[
                    Point2::new(x, BOUNDS.y + 0.2 * BOUNDS.h),
                    Point2::new(x, BOUNDS.y + 0.8 * BOUNDS.h),
                ],
                4.0,
                graphics::WHITE,
            )?;
            graphics::draw(ctx, &line, DrawParam::default())?;
        }
        Ok(())
    }
}
