use std::path::{Path, PathBuf};

use anyhow::Result;
use ggez::{
    graphics::{self, DrawParam, Image},
    nalgebra::Point2,
    Context,
};

macro_rules! create_skin {
    ($([$name:ident]),* $(,)?) => {
        pub struct Skin {
            $(
                pub $name: Texture,
             )*
        }

        impl Skin {
            pub fn new() -> Self {
                Skin {
                    $($name: Texture::with_path(concat!("/", stringify!($name), ".png")),)*
                }
            }
        }
    }
}

create_skin! {
    [approachcircle],
    [hitcircle],
    [hitcircleoverlay],
}

pub struct Texture {
    path: PathBuf,
    image: Option<Image>,
}

impl Texture {
    pub fn with_path(path: impl AsRef<Path>) -> Self {
        Texture {
            path: path.as_ref().to_path_buf(),
            image: None,
        }
    }

    pub fn draw(&mut self, ctx: &mut Context, size: (f32, f32), param: DrawParam) -> Result<()> {
        let image = if self.image.is_some() {
            self.image.as_ref().unwrap()
        } else {
            self.image = Some(Image::new(ctx, &self.path)?);
            self.image.as_ref().unwrap()
        };

        let random_constant = 1.35;
        let x_scale = random_constant * size.0 / image.width() as f32;
        let y_scale = random_constant * size.1 / image.height() as f32;
        graphics::draw(
            ctx,
            image,
            param
                .scale([x_scale, y_scale])
                .offset(Point2::new(0.5, 0.5)),
        )?;
        Ok(())
    }
}
