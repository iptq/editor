use anyhow::Result;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

pub struct ObjectWrapper {
    id: usize,
    inner: Box<dyn Object>,
}

pub trait Object {
    fn update(&mut self);

    fn draw(&self);
}

pub struct Context {}

pub struct Game {
    event_loop: EventLoop<()>,
    window: Window,
    objects: Vec<ObjectWrapper>,
}

impl Game {
    pub fn init() -> Result<Self> {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new().build(&event_loop)?;

        Ok(Game {
            objects: vec![],
            event_loop,
            window,
        })
    }

    pub fn run(mut self) {
        let window_id = self.window.id();
        self.event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    window_id,
                } if window_id == window_id => *control_flow = ControlFlow::Exit,
                _ => (),
            }
        });
    }
}
