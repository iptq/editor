use std::mem::ManuallyDrop;
use std::ptr;

use anyhow::{anyhow, Result};
use gfx_hal::{
    adapter::{Adapter, Gpu, PhysicalDevice},
    device::Device,
    pool::CommandPoolCreateFlags,
    queue::{family::QueueFamily, QueueGroup, QueueType},
    window::Surface,
    Backend, Features, Instance,
};
use winit::window::Window;

pub struct Renderer<B: Backend> {
    surface: ManuallyDrop<B::Surface>,
    device: B::Device,
    adapter: Adapter<B>,
    queue_group: QueueGroup<B>,
    instance: B::Instance,
}

impl<B: Backend> Renderer<B> {
    pub fn new(
        instance: B::Instance,
        mut surface: B::Surface,
        adapter: Adapter<B>,
    ) -> Result<Renderer<B>> {
        let family = adapter
            .queue_families
            .iter()
            .find(|family| {
                surface.supports_queue_family(family) && family.queue_type().supports_graphics()
            })
            .unwrap();

        let mut gpu = unsafe {
            adapter
                .physical_device
                .open(&[(family, &[1.0])], Features::empty())
        }?;

        let mut queue_group = gpu.queue_groups.pop().unwrap();
        let device = gpu.device;

        let mut command_pool = unsafe {
            device.create_command_pool(queue_group.family, CommandPoolCreateFlags::empty())
        }?;

        Ok(Renderer {
            surface: ManuallyDrop::new(surface),
            device,
            adapter,
            queue_group,
            instance,
        })
    }
}

impl<B: Backend> Drop for Renderer<B> {
    fn drop(&mut self) {
        self.device.wait_idle().unwrap();

        unsafe {
            let surface = ManuallyDrop::into_inner(ptr::read(&self.surface));
            self.instance.destroy_surface(surface);
        }
    }
}
