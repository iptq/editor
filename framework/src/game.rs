#[cfg(windows)]
pub use gfx_backend_dx12 as back;
#[cfg(target_arch = "wasm32")]
pub use gfx_backend_gl as back;
#[cfg(target_os = "macos")]
pub use gfx_backend_metal as back;
#[cfg(not(any(windows, target_os = "macos", target_arch = "wasm32")))]
pub use gfx_backend_vulkan as back;

use anyhow::Result;
use gfx_hal::Instance;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use crate::graphics::Renderer;

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

        #[cfg(target_arch = "wasm32")]
        web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .body()
            .unwrap()
            .append_child(&winit::platform::web::WindowExtWebSys::canvas(&window))
            .unwrap();

        // let instance = back::Instance::create("osu", 1).unwrap();
        // let surface = unsafe { instance.create_surface(&window) }?;
        // let adapter = {
        //     let mut adapters = instance.enumerate_adapters();
        //     for adapter in adapters.iter() {
        //         println!("{:?}", adapter.info);
        //     }
        //     adapters.remove(0)
        // };
        // let renderer = Renderer::new(instance, surface, adapter)?;

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
