use anyhow::Result;
use ggez::{
    graphics::{self, DrawParam, Image},
    Context,
};

macro_rules! create_skin {
    (
        // regular skin textures
        $( [$name:ident, $path:expr, $animatable:expr $(,)?]),*
        $(,)?
    ) => {
        pub struct Skin {
            $(
                pub $name: Texture,
             )*
        }

        impl Skin {
            pub fn new() -> Self {
                Skin {
                    $($name: Texture::with_name(stringify!($name), $path, $animatable),)*
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
    [approachcircle, "approachcircle", false],
    [hitcircle, "hitcircle", false],
    [hitcircleoverlay, "hitcircleoverlay", false],
    [reversearrow, "reversearrow", false],
    [sliderb, "sliderb", true],

    // TODO: actually read numbers from skin.ini
    [default0, "default-0", false],
    [default1, "default-1", false],
    [default2, "default-2", false],
    [default3, "default-3", false],
    [default4, "default-4", false],
    [default5, "default-5", false],
    [default6, "default-6", false],
    [default7, "default-7", false],
    [default8, "default-8", false],
    [default9, "default-9", false],
}

pub struct Texture {
    name: &'static str,
    path: &'static str,
    image: Option<Image>,
    animatable: bool,
    animation: Vec<Image>,
}

impl Texture {
    pub fn with_name(name: &'static str, path: &'static str, animatable: bool) -> Self {
        Texture {
            name,
            path,
            image: None,
            animatable,
            animation: vec![],
        }
    }

    pub fn width(&self) -> Option<u16> {
        self.image.as_ref().map(|image| image.width())
    }

    pub fn height(&self) -> Option<u16> {
        self.image.as_ref().map(|image| image.height())
    }

    pub fn load(&mut self, ctx: &mut Context) -> Result<()> {
        let mut found = false;
        if self.animatable {
            // god fucking dammit
            let hyphen = if self.name == "sliderb" { "" } else { "-" };

            self.animation.clear();
            let mut curr = 0;
            while let Ok(image) = Image::new(ctx, &format!("/{}{}{}.png", self.path, hyphen, curr))
            {
                self.animation.push(image);
                found = true;
                curr += 1;
            }
            println!("loaded {} images!", curr);
        }

        if !found {
            self.image = Some(Image::new(ctx, &format!("/{}.png", self.path))?);
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
            param.scale([x_scale, y_scale]).offset([0.5, 0.5]),
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
