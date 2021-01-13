use anyhow::Result;
use ggez::{
    graphics::{self, Color, DrawParam},
    Context,
};

use super::Game;

impl Game {
    pub(super) fn draw_background(&self, ctx: &mut Context) -> Result<()> {
        let screen_width = 1024.0;
        let screen_height = 768.0;
        let screen_ratio = screen_width / screen_height;

        if let Some(image) = &self.background_image {
            let dim = Color::new(1.0, 1.0, 1.0, 0.35);
            let width = image.width() as f32;
            let height = image.height() as f32;
            let ratio = width / height;

            let (scale, offset) = if ratio < screen_ratio {
                // background is flatter than screen
                let scale = screen_width / width;
                let real_height = scale * height;
                let diff = real_height - screen_height;
                ([scale, scale], [0.0, -diff / 2.0])
            } else if ratio > screen_ratio {
                // background is more square than screen
                // take screen height
                let scale = screen_height / height;
                let real_width = scale * width;
                let diff = real_width - screen_width;
                ([scale, scale], [-diff / 2.0, 0.0])
            } else {
                // exactly the same ratio
                let scale = [screen_width / width, screen_height / height];
                let offset = [0.0, 0.0];
                (scale, offset)
            };

            graphics::draw(
                ctx,
                image,
                DrawParam::default().color(dim).scale(scale).dest(offset),
            )?;
        }

        Ok(())
    }
}
