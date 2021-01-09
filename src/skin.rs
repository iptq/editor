use anyhow::Result;
use ggez::{
    graphics::{self, DrawParam, Image},
    nalgebra::Point2,
    Context,
};

macro_rules! create_skin {
    ($([$name:ident, $animatable:expr $(,)?]),* $(,)?) => {
        pub struct Skin {
            $(
                pub $name: Texture,
             )*
        }

        impl Skin {
            pub fn new() -> Self {
                Skin {
                    $($name: Texture::with_name(stringify!($name), $animatable),)*
                }
            }

            // TODO: do this asynchronously?
            pub fn load_all(&mut self, ctx: &mut Context) -> Result<()> {
                $(
                    self.$name.load(ctx)?;
                 )*
                Ok(())
            }
        }
    }
}

create_skin! {
    [approachcircle, false],
    [hitcircle, false],
    [hitcircleoverlay, false],
    [reversearrow, false],
    [sliderb, true],
}

pub struct Texture {
    name: &'static str,
    image: Option<Image>,
    animatable: bool,
    animation: Vec<Image>,
}

impl Texture {
    pub fn with_name(name: &'static str, animatable: bool) -> Self {
        Texture {
            name,
            image: None,
            animatable,
            animation: vec![],
        }
    }

    pub fn load(&mut self, ctx: &mut Context) -> Result<()> {
        let mut found = false;
        if self.animatable {
            // god fucking dammit
            let hyphen = if self.name == "sliderb" { "" } else { "-" };

            self.animation.clear();
            let mut curr = 0;
            loop {
                let image = match Image::new(ctx, &format!("/{}{}{}.png", self.name, hyphen, curr))
                {
                    Ok(v) => v,
                    Err(_) => break,
                };
                self.animation.push(image);
                found = true;
                curr += 1;
            }
            println!("loaded {} images!", curr);
        }

        if !found {
            self.image = Some(Image::new(ctx, &format!("/{}.png", self.name))?);
        }

        Ok(())
    }

    fn draw_image(
        &self,
        ctx: &mut Context,
        image: &Image,
        size: (f32, f32),
        param: DrawParam,
    ) -> Result<()> {
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

    pub fn draw(&self, ctx: &mut Context, size: (f32, f32), param: DrawParam) -> Result<()> {
        let image = self.image.as_ref().unwrap();
        self.draw_image(ctx, image, size, param)
    }

    pub fn draw_frame(
        &self,
        ctx: &mut Context,
        size: (f32, f32),
        param: DrawParam,
        frame: usize,
    ) -> Result<()> {
        let image = if self.animatable {
            let ct = frame % self.animation.len();
            self.animation.get(ct).unwrap()
        } else {
            self.image.as_ref().unwrap()
        };
        self.draw_image(ctx, image, size, param)
    }
}
