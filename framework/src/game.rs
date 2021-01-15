pub struct ObjectWrapper {
    id: usize,
    inner: Box<dyn Object>,
}

pub trait Object {
    fn update(&mut self);

    fn draw(&self);
}

pub struct Context {
}

pub struct Game {
    objects: Vec<ObjectWrapper>,
}

impl Game {
    pub fn run<F>(mut self, func: F)
    where
        F: Fn(),
    {
        loop {
            for object in self.objects.iter_mut() {
            }
        }
    }
}
