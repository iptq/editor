use anyhow::Result;
use ggez::{
    graphics::{self, DrawParam},
    Context,
};

use super::Game;

impl Game {
    pub(super) fn draw_numbers_on_circle(
        &self,
        ctx: &mut Context,
        number: usize,
        pos: [f32; 2],
        cs: f32,
    ) -> Result<()> {
        let number = number.to_string();
        let digits = number.len();
        let cs = cs / 1.5;

        let mut digits = Vec::new();
        let mut width = 0;
        let mut first_height = None;
        let spacing = 5;
        for digit in number.chars() {
            let texture = match digit {
                '0' => &self.skin.default0,
                '1' => &self.skin.default1,
                '2' => &self.skin.default2,
                '3' => &self.skin.default3,
                '4' => &self.skin.default4,
                '5' => &self.skin.default5,
                '6' => &self.skin.default6,
                '7' => &self.skin.default7,
                '8' => &self.skin.default8,
                '9' => &self.skin.default9,
                _ => unreachable!(),
            };
            if let None = first_height {
                first_height = Some(texture.height().unwrap());
            }
            let this_width = texture.width().unwrap();
            width += this_width + spacing;
            digits.push((digit, width, this_width, texture));
        }

        let height = first_height.unwrap();
        let real_total_width = cs * width as f32 / height as f32;
        let real_height = cs;
        let left_off = pos[0] - real_total_width;
        let real_y = pos[1];

        for (_, x, w, digit) in digits {
            let w = w as f32 / height as f32 * cs;
            let real_x = left_off + x as f32 / width as f32 * real_total_width;
            digit.draw(
                ctx,
                (w, real_height),
                DrawParam::default().dest([real_x, real_y]),
            )?;
        }

        // let pos = [leftmost + i as f32 / 2.0 * digit_width, pos[1]];
        // let rect = graphics::Mesh::new_rectangle(
        //     ctx,
        //     graphics::DrawMode::Stroke(graphics::StrokeOptions::default()),
        //     [pos[0], pos[1], digit_width, cs].into(),
        //     [1.0, 0.0, 0.0, 1.0].into(),
        // )?;
        // graphics::draw(ctx, &rect, DrawParam::default().offset([0.5, 0.5]))?;
        // texture.draw(
        //     ctx,
        //     (cs * 42.0 / 72.0, cs),
        //     DrawParam::default().offset([0.5, 0.5]).dest(pos),
        // )?;
        Ok(())
    }
}
