use anyhow::Result;
use ggez::{
    conf::NumSamples,
    graphics::{self, Canvas, Color, DrawMode, DrawParam, FillOptions, Mesh, Rect},
    mint::Point2,
    Context,
};
use libosu::timing::TimingPointKind;

use super::Game;

pub const BOUNDS: Rect = Rect::new(46.0, 732.0, 932.0, 36.0);

impl Game {
    pub(super) fn draw_seeker(&mut self, ctx: &mut Context) -> Result<()> {
        if self.seeker_cache.is_none() {
            println!("drawing seeker");
            let format = graphics::get_window_color_format(ctx);
            let canvas = Canvas::new(
                ctx,
                BOUNDS.w as u16,
                BOUNDS.h as u16,
                NumSamples::Sixteen,
                format,
            )?;
            graphics::set_canvas(ctx, Some(&canvas));

            let rect = Mesh::new_rectangle(
                ctx,
                DrawMode::Fill(FillOptions::default()),
                Rect::new(0.0, 732.0, 1024.0, 36.0),
                Color::new(0.0, 0.0, 0.0, 0.7),
            )?;
            graphics::draw(ctx, &rect, DrawParam::default())?;

            let line_y = BOUNDS.h / 2.0;
            let line = Mesh::new_line(
                ctx,
                &[
                    Point2::from([0.0, line_y]),
                    Point2::from([BOUNDS.w, line_y]),
                ],
                1.0,
                Color::WHITE,
            )?;
            graphics::draw(ctx, &line, DrawParam::default())?;

            if let Some(song) = &self.song {
                let len = song.length()?;

                for timing_point in self.beatmap.inner.timing_points.iter() {
                    let color = match timing_point.kind {
                        TimingPointKind::Inherited(_) => Color::new(0.0, 0.8, 0.0, 0.4),
                        TimingPointKind::Uninherited(_) => Color::new(0.8, 0.0, 0.0, 0.6),
                    };

                    let percent = timing_point.time.as_seconds() / len;
                    let x = percent as f32 * BOUNDS.w;

                    let line = Mesh::new_line(
                        ctx,
                        &[Point2::from([x, 0.0]), Point2::from([x, BOUNDS.h / 2.0])],
                        1.0,
                        color,
                    )?;
                    graphics::draw(ctx, &line, DrawParam::default())?;
                }

                let percent = song.position()? / len;
                let x = percent as f32 * BOUNDS.w;
                let line = Mesh::new_line(
                    ctx,
                    &[
                        Point2::from([x, 0.2 * BOUNDS.h]),
                        Point2::from([x, 0.8 * BOUNDS.h]),
                    ],
                    4.0,
                    Color::WHITE,
                )?;
                graphics::draw(ctx, &line, DrawParam::default())?;
            }

            graphics::set_canvas(ctx, None);
            self.seeker_cache = Some(canvas);
        };

        if let Some(canvas) = &self.seeker_cache {
            graphics::draw(
                ctx,
                canvas,
                DrawParam::default()
                    .dest([BOUNDS.x, BOUNDS.y])
                    .scale([1.0, 10.0]),
            )?;
        }
        Ok(())
    }
}
