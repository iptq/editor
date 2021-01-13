use anyhow::Result;
use ggez::{
    graphics::{self, Color, DrawParam},
    Context,
};

use super::Game;

impl Game {
    pub(super) fn draw_background(&self, ctx: &mut Context) -> Result<()> {
        let screen_width = 1024;
        let screen_height = 768;
        let screen_ratio = screen_width as f32 / screen_height as f32;

        if let Some(image) = &self.background_image {
            let dim = Color::new(1.0, 1.0, 1.0, 0.35);
            let width = image.width();
            let height = image.height();
            let ratio = width as f32 / height as f32;

            let scale =
            // background is flatter than screen
            if ratio < screen_ratio {
                // take screen height
                [screen_width as f32/width as f32,screen_height as f32/height as f32]
            }
            // background is more square than screen
            else if ratio > screen_ratio {
                [screen_width as f32/width as f32,screen_height as f32/height as f32]
            }
            // exactly the same ratio
            else {
                [screen_width as f32/width as f32,screen_height as f32/height as f32]
            };
            graphics::draw(ctx, image, DrawParam::default().color(dim).scale(scale))?;
        }

        Ok(())
    }
}
