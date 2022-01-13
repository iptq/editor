use anyhow::Result;
use ggez::{
    graphics::{self, Color, DrawMode, DrawParam, FillOptions, Mesh, Rect},
    mint::Point2,
    Context,
};
use libosu::timing::TimingPointKind;

use super::Game;

pub const BOUNDS: Rect = Rect::new(46.0, 732.0, 932.0, 36.0);

impl Game {
    pub(super) fn draw_seeker(&mut self, ctx: &mut Context) -> Result<()> {
        let rect = Mesh::new_rectangle(
            ctx,
            DrawMode::Fill(FillOptions::default()),
            Rect::new(0.0, 732.0, 1024.0, 36.0),
            Color::new(0.0, 0.0, 0.0, 0.7),
        )?;
        graphics::draw(ctx, &rect, DrawParam::default())?;

        // draw the main timeline of the seeker
        let line_y = BOUNDS.y + BOUNDS.h / 2.0;
        let line = Mesh::new_line(
            ctx,
            &[
                Point2::from([BOUNDS.x, line_y]),
                Point2::from([BOUNDS.w, line_y]),
            ],
            1.0,
            Color::WHITE,
        )?;
        graphics::draw(ctx, &line, DrawParam::default())?;

        if let Some(song) = &self.song {
            let len = song.length()?;

            // draw timing points
            for timing_point in self.beatmap.inner.timing_points.iter() {
                let color = match timing_point.kind {
                    TimingPointKind::Inherited(_) => Color::new(0.0, 0.8, 0.0, 0.4),
                    TimingPointKind::Uninherited(_) => Color::new(0.8, 0.0, 0.0, 0.6),
                };

                let percent = timing_point.time.as_seconds() / len;
                let x = BOUNDS.x + percent as f32 * BOUNDS.w;

                let line = Mesh::new_line(
                    ctx,
                    &[
                        Point2::from([x, BOUNDS.y]),
                        Point2::from([x, BOUNDS.y + BOUNDS.h / 2.0]),
                    ],
                    1.0,
                    color,
                )?;
                graphics::draw(ctx, &line, DrawParam::default())?;
            }

            // draw the knob for current position
            let percent = song.position()? / len;
            let x = BOUNDS.x + percent as f32 * BOUNDS.w;
            let line = Mesh::new_line(
                ctx,
                &[
                    Point2::from([x, BOUNDS.y + 0.2 * BOUNDS.h]),
                    Point2::from([x, BOUNDS.y + 0.8 * BOUNDS.h]),
                ],
                4.0,
                Color::WHITE,
            )?;
            graphics::draw(ctx, &line, DrawParam::default())?;
        }

        Ok(())
    }
}
