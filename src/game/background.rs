use anyhow::Result;
use ggez::{
    graphics::{self, Color, DrawParam},
    Context,
};

use super::Game;

impl Game {
    pub(super) fn draw_background(&self, ctx: &mut Context) -> Result<()> {
        if let Some(image) = &self.background_image {
            let dim = Color::new(1.0, 1.0, 1.0, 0.35);
            graphics::draw(ctx, image, DrawParam::default().color(dim))?;
        }

        Ok(())
    }
}
