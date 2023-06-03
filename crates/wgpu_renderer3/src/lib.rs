pub mod vertex_data;

pub mod pipelines;
pub use pipelines::generic_pipeline::Pipeline2D;

use euclid::default::Size2D;

use std::sync::Mutex;
use std::time::Duration;


#[derive(Default)]
pub struct ActualFrameInfo {
    frame: Option<wgpu::SwapChainFrame>,
}

#[allow(dead_code)]
pub struct UniformBuffer {
    camera_x: f32,
    camera_y: f32,
    camera_z: f32,
    window_width: f32,
    window_height: f32,
}

#[allow(dead_code)]
pub struct Renderer {
    adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    surface: wgpu::Surface,
    swap_chain: wgpu::SwapChain,
    swap_chain_size: Size2D<u32>,
    actual_frame: ActualFrameInfo,
    present_mode: wgpu::PresentMode,
    present_mode_changed: Mutex<Option<wgpu::PresentMode>>,
    resize_events: Mutex<Option<Size2D<u32>>>,
    fps: Fps,
}

impl Renderer {
    pub fn new(window: &dyn std::any::Any, size: Size2D<u32>, present_mode: wgpu::PresentMode) -> Result<Renderer, Box<dyn std::error::Error>> {
        let instance = wgpu::Instance::new(wgpu::BackendBit::VULKAN);
        let surface = {
            let window = unsafe { std::mem::transmute::<&dyn std::any::Any, &'static dyn std::any::Any>(window) };
            if let Some(window) = window.downcast_ref::<winit::window::Window>() { unsafe { instance.create_surface(window) } }
            else { panic!(); }
        };
        let adapter = futures_executor::block_on(instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface)
            },
        )).ok_or("No vulkan card detected")?;
        let (device, queue) = futures_executor::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::SAMPLED_TEXTURE_BINDING_ARRAY,
                limits: wgpu::Limits::default(),
            },
            None
        ))?;
        let swap_chain_size = Size2D::new(size.width, size.height);
        let swap_chain = device.create_swap_chain(&surface, &wgpu::SwapChainDescriptor {                
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8Unorm,
            width: swap_chain_size.width,
            height: swap_chain_size.height,
            present_mode: present_mode,
        });
        
        let fps = Fps::new();
        Ok(Renderer {
            adapter,
            device,
            queue,
            surface,
            swap_chain,
            swap_chain_size,
            actual_frame: ActualFrameInfo::default(),
            present_mode: present_mode,
            present_mode_changed: Mutex::new(None),
            resize_events: Mutex::new(None),
            fps,
        })
    }

    pub fn get_device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn get_queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    pub fn new_frame(&mut self) -> Result<&wgpu::SwapChainFrame, ()> {
        self.update_swapchain()?;
        self.actual_frame.frame = self.swap_chain.get_current_frame().ok();
        self.fps.new_frame();
        if let Some(frame) = self.actual_frame.frame.as_ref() {
            Ok(frame)
        } else {
            Err(())
        }
    }

    pub fn get_actual_frame(&self) -> Result<&wgpu::SwapChainFrame, ()> {
        self.actual_frame.frame.as_ref().ok_or(())
    }

    pub fn end_frame(&mut self) {
        self.actual_frame.frame.take();
    }

    pub fn resize_window(&self, size: Size2D<u32>) {
        *self.resize_events.lock().unwrap() = Some(size);
    }

    pub fn get_present_mode(&self) -> wgpu::PresentMode {
        self.present_mode
    }

    pub fn set_present_mode(&self, present_mode: wgpu::PresentMode) {
        *self.present_mode_changed.lock().unwrap() = Some(present_mode);
    }

    fn update_swapchain(&mut self) -> Result<(), ()> {
        let mut new_size_window = self.resize_events.lock().unwrap();
        let mut new_present_mode = self.present_mode_changed.lock().unwrap();
        if new_size_window.is_none() && new_present_mode.is_none() { return Ok(()) }
        
        let size_window = new_size_window.unwrap_or(self.swap_chain_size);
        let present_mode = new_present_mode.unwrap_or(self.present_mode);
        
        if size_window.width as u32 == 0 || size_window.height as u32 == 0 {
            return Err(())
        }
        self.swap_chain_size = size_window;
        self.present_mode = present_mode;

        self.swap_chain = self.device.create_swap_chain(&self.surface, &wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8Unorm,
            width: self.swap_chain_size.width as u32,
            height: self.swap_chain_size.height as u32,
            present_mode: present_mode,
        });

        *new_size_window = None;
        *new_present_mode = None;

        Ok(())
    }

    pub fn get_fps(&mut self, duration: Duration) -> i32 {
        self.fps.get_fps(duration)
    }
}
pub struct Fps {
    frames: Vec<std::time::Instant>,
}

impl<'a> Fps {
    fn new() -> Fps {
        Fps {
            frames: Vec::new(),
        }
    }

    fn new_frame(&mut self) {
        self.frames.push(std::time::Instant::now());
        self.truncate();
    }

    fn get_fps(&self, duration: Duration) -> i32 {
        self.frames.iter()
            .rev()
            .filter(|instant| instant.elapsed() < duration)
            .count() as i32
    }

    fn truncate(&mut self) {
        let frames = self.frames.iter()
            .cloned()
            .filter(|instant| instant.elapsed() < Duration::from_secs(5))
            .collect();
        
        self.frames = frames;
    }
}