use anyhow::Result;
use ggez::{
    graphics::{self, Color, DrawMode, DrawParam, Mesh, StrokeOptions},
    mint::Point2,
    Context,
};

use super::{Game, PLAYFIELD_BOUNDS as BOUNDS};

pub const GRID_HEAVY: Color = Color::new(1.0, 1.0, 1.0, 0.25);
pub const GRID_LIGHT: Color = Color::new(1.0, 1.0, 1.0, 0.05);

impl Game {
    pub(super) fn draw_grid(&self, ctx: &mut Context) -> Result<()> {
        let playfield = Mesh::new_rectangle(
            ctx,
            DrawMode::Stroke(StrokeOptions::default()),
            BOUNDS,
            Color::new(1.0, 1.0, 1.0, 0.5),
        )?;
        graphics::draw(ctx, &playfield, DrawParam::default())?;

        let min_x = BOUNDS.x;
        let min_y = BOUNDS.y;
        let max_x = min_x + BOUNDS.w;
        let max_y = min_y + BOUNDS.h;

        let grid_size = self.beatmap.inner.grid_size;

        for x in (0..512).step_by(grid_size as usize) {
            let (weight, color) = match x {
                x if x == 0 || x == 256 || x == 512 => (2.0, GRID_HEAVY),
                _ => (1.0, GRID_LIGHT),
            };
            let x = BOUNDS.x + x as f32 * BOUNDS.w / 512.0;
            let line = Mesh::new_line(
                ctx,
                &[Point2::from([x, min_y]), Point2::from([x, max_y])],
                weight,
                color,
            )?;
            graphics::draw(ctx, &line, DrawParam::default())?;
        }

        for y in (0..384).step_by(grid_size as usize) {
            let (weight, color) = match y {
                y if y == 0 || y == 192 || y == 384 => (2.0, GRID_HEAVY),
                _ => (1.0, GRID_LIGHT),
            };
            let y = BOUNDS.y + y as f32 * BOUNDS.h / 384.0;
            let line = Mesh::new_line(
                ctx,
                &[Point2::from([min_x, y]), Point2::from([max_x, y])],
                weight,
                color,
            )?;
            graphics::draw(ctx, &line, DrawParam::default())?;
        }

        Ok(())
    }

    pub(super) fn toggle_grid(&mut self) {
        use libosu::enums::GridSize::*;
        self.beatmap.inner.grid_size = match self.beatmap.inner.grid_size {
            Tiny => Small,
            Small => Medium,
            Medium => Large,
            Large => Tiny,
        };
    }
}
