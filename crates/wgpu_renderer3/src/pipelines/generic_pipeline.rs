use crate::pipelines::ssaa::{SSAA, SSAAFactor};
use crate::pipelines::renderer_pipeline_colored_i32::RendererPipelineColoredi32;
use crate::pipelines::renderer_pipeline_textured_i32::RendererPipelineTexturedi32;
use crate::pipelines::pipeline_compute_vertex_i32::PipelineComputeVertexi32;
use crate::Renderer;
use crate::vertex_data::Vertex2DColoredi32Buffer;
use crate::vertex_data::Vertex2DTexturedi32Buffer;
use crate::vertex_data::Vertex2DTexturedi32ComputeBuffer;
use euclid::default::Size2D;

pub trait ComputePreVertexPipeline2D {
    fn bind_group(&self) -> &wgpu::BindGroup;
    fn buffer_input(&self) -> &wgpu::Buffer;
    fn buffer_input_size(&self) -> u64;
    fn buffer_output(&self) -> &wgpu::Buffer;
    fn buffer_output_size(&self) -> u64;
    fn pipeline(&self) -> &wgpu::ComputePipeline;
    fn size_data(&self) -> usize;
}
pub trait RendererPipeline2D {
    fn bind_group(&self) -> &wgpu::BindGroup;
    fn pipeline(&self) -> &wgpu::RenderPipeline;
    fn vertex_size(&self) -> usize;
}

#[derive(Default)]
pub struct VertexBufferSize {
    vertex_count: usize, 
    vertex_size: usize,
}

#[derive(Clone)]
pub struct Camera2D {
    pub x: i32,
    pub y: i32,
    pub size: f32,
}

impl Camera2D {
    pub fn position_world(&self, position: (i32, i32), size_window: (i32, i32)) -> Option<(i32, i32)> {
        if size_window.0 == 0 || size_window.1 == 0 { return None }

        let position = euclid::default::Point2D::new(position.0, position.1);
        let size_window = euclid::default::Size2D::new(size_window.0, size_window.1);

        let position_window_normalized = position - euclid::default::Vector2D::new(size_window.width as i32 / 2, size_window.height as i32 / 2);
        let position_camera_z = (position_window_normalized.to_f32() * euclid::default::Scale::new(self.size)).to_i32();
        let position_world = position_camera_z + euclid::default::Vector2D::new(self.x, self.y);

        return Some((position_world.x, position_world.y));
    }
}

#[allow(dead_code)]
struct UniformBuffer{
    camera: Camera2D,
    window_width: f32,
    window_height: f32,
    ssaa: i32,
}

#[derive(Clone)]
pub enum ClearInfo {
    All(wgpu::Color),
    Frame(wgpu::Color),
    Depth,
}

pub struct Pipeline2D {
    pub swap_chain_size: Size2D<u32>,
    pub vertex_buffer: wgpu::Buffer,
    pub vertex_buffer_size: wgpu::BufferAddress,
    pub vertex_buffer_actual_frame_size: VertexBufferSize,
    pub uniform_buffer: wgpu::Buffer,
    pub mapped_buffer_size: wgpu::BufferAddress,
    pub mapped_buffer: wgpu::Buffer,
    pub mapped_buffer_cpu_access: Option<wgpu::BufferViewMut<'static>>,
    pub depth_buffer_texture: wgpu::Texture,
    pub render_pipeline_colored_i32: RendererPipelineColoredi32,
    pub render_pipeline_textured_i32: RendererPipelineTexturedi32,
    pub compute_pipeline_vertex_i32: PipelineComputeVertexi32,
    ssaa: Option<SSAA>,
}

impl Pipeline2D {
    pub fn new(core: &crate::Renderer) -> Pipeline2D {
        let swap_chain_size = core.swap_chain_size;
        let vertex_buffer_size = 5_000_000;
        let vertex_buffer = core.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            mapped_at_creation: false,
            size: vertex_buffer_size,
            usage: wgpu::BufferUsage::COPY_DST | wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::INDEX,
        });
        let vertex_buffer_actual_frame_size = VertexBufferSize::default();
        let uniform_buffer = core.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            mapped_at_creation: false,
            size: 64,
            usage: wgpu::BufferUsage::COPY_DST | wgpu::BufferUsage::UNIFORM,
        });
        let mapped_buffer_size = 10_000_000;
        let mapped_buffer = core.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            mapped_at_creation: false,
            size: mapped_buffer_size,
            usage: wgpu::BufferUsage::COPY_SRC | wgpu::BufferUsage::MAP_WRITE,
        });
        let mapped_buffer_cpu_access = None;
        
        let depth_buffer_texture = core.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {                    
                width: swap_chain_size.width as u32,
                height: swap_chain_size.height as u32,
                depth: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth24Plus,
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
        });

        let _vertex_shader_colored_f32 = core.device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::util::make_spirv(&include_bytes!("../../shaders/2D_colored/compiled/vert_f32.spv")[..]),
            flags: wgpu::ShaderFlags::empty(),
        });
        let _vertex_shader_textured_f32 = core.device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::util::make_spirv(&include_bytes!("../../shaders/2D_textured/compiled/vert_f32.spv")[..]),
            flags: wgpu::ShaderFlags::empty(),
        });
        let _fragment_shader_textured = core.device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::util::make_spirv(&include_bytes!("../../shaders/2D_textured/compiled/frag.spv")[..]),
            flags: wgpu::ShaderFlags::empty(),
        });

        let render_pipeline_colored_i32 = crate::pipelines::renderer_pipeline_colored_i32::RendererPipelineColoredi32::new(core, &uniform_buffer);
        let render_pipeline_textured_i32 = crate::pipelines::renderer_pipeline_textured_i32::RendererPipelineTexturedi32::new(core, &uniform_buffer);
        let compute_pipeline_vertex_i32 = crate::pipelines::pipeline_compute_vertex_i32::PipelineComputeVertexi32::new(core);

        let ssaa = None;

        Pipeline2D {
            swap_chain_size,
            vertex_buffer,
            vertex_buffer_size,
            vertex_buffer_actual_frame_size,
            uniform_buffer,
            mapped_buffer_size,
            mapped_buffer,
            mapped_buffer_cpu_access,
            depth_buffer_texture,
            render_pipeline_colored_i32,
            render_pipeline_textured_i32,
            compute_pipeline_vertex_i32,
            ssaa,
        }
    }

    pub fn update_swapchain(&mut self, core: &Renderer) {
        if self.swap_chain_size != core.swap_chain_size {
            self.swap_chain_size = core.swap_chain_size;
            self.depth_buffer_texture = core.device.create_texture(&wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d {                    
                    width: self.swap_chain_size.width as u32,
                    height: self.swap_chain_size.height as u32,
                    depth: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth24Plus,
                usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            });
        }
    }

    fn update_uniform_buffer(&mut self, core: &Renderer, camera: &Camera2D) {    
        let uniform_buffer = UniformBuffer {
            camera: camera.clone(),
            window_width: core.swap_chain_size.width as f32,
            window_height: core.swap_chain_size.height as f32,
            ssaa: self.get_ssaa_mul(),
        };
        let uniform_buffer_array = [uniform_buffer];

        let uniform_buffer_raw = unsafe { uniform_buffer_array.align_to::<u8>().1 };
        /*
        let staging_buffer = core.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: uniform_buffer_raw.len() as u64,
            usage: wgpu::BufferUsage::MAP_WRITE | wgpu::BufferUsage::COPY_SRC,
            mapped_at_creation: true,
        });

        let slice = staging_buffer.slice(..);
        {
            let mut mapped_buffer = slice.get_mapped_range_mut();
            mapped_buffer.copy_from_slice(&uniform_buffer_raw);
        }
        staging_buffer.unmap();

        let mut command_encoder = core.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        command_encoder.copy_buffer_to_buffer(&staging_buffer, 0, &self.uniform_buffer, 0, std::mem::size_of::<UniformBuffer>() as u64);
        core.queue.submit(vec![command_encoder.finish()]);
        */

        core.get_queue().write_buffer(&self.uniform_buffer, 0, uniform_buffer_raw);
    }

    pub fn set_mapped_buffer_size(&mut self, core: &Renderer, size: wgpu::BufferAddress) {
        //if size <= self.mapped_buffer_size { return }
        self.mapped_buffer_cpu_access = None; //Unmap buffer if already mapped
        self.mapped_buffer_size = size;
        self.mapped_buffer = core.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            mapped_at_creation: false,
            size: self.mapped_buffer_size,
            usage: wgpu::BufferUsage::COPY_SRC | wgpu::BufferUsage::MAP_WRITE,
        });
    }

    pub fn set_vertex_buffer_size(&mut self, core: &Renderer, size: wgpu::BufferAddress) {
        //if size <= self.vertex_buffer_size { return }
        self.vertex_buffer_size = size;
        self.vertex_buffer = core.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            mapped_at_creation: false,
            size: self.vertex_buffer_size,
            usage: wgpu::BufferUsage::COPY_DST | wgpu::BufferUsage::VERTEX,
        });
    }

    pub fn get_ssaa_image(&self) -> &wgpu::Texture {
        &self.ssaa.as_ref().unwrap().texture_frame
    }

    pub fn get_ssaa_image_view(&self) -> &wgpu::TextureView {
        &self.ssaa.as_ref().unwrap().texture_frame_view
    }

    pub fn update_ssaa(&mut self, core:&Renderer) {
        self.ssaa.as_mut().unwrap().draw(core);
    }

    pub fn get_ssaa_mul(&self) -> i32 {
        if let Some(ssaa) = self.ssaa.as_ref() {
            match ssaa.get_ssaa_factor() {
                SSAAFactor::X2 => { 2 }
                SSAAFactor::X4 => { 4 }
                SSAAFactor::X8 => { 8 }
                _ => { panic!("Bug SSAA 2") }
            }
        } else {
            1
        }
    }

    pub fn set_ssaa(&mut self, core: &Renderer, ratio: SSAAFactor) {
        match ratio {
            SSAAFactor::Disabled => {self.ssaa = None;}
            SSAAFactor::X2 => {self.ssaa = Some(SSAA::new(core, 2));}
            SSAAFactor::X4 => {self.ssaa = Some(SSAA::new(core, 3));}
            SSAAFactor::X8 => {self.ssaa = Some(SSAA::new(core, 4));}
        }

        if let Some(_ssaa) = self.ssaa.as_ref() {
            self.clear_ssaa(core, ClearInfo::All(wgpu::Color::BLACK));
        }
    }

    pub fn draw_colored_i32(&mut self, core: &Renderer, vertex_data: &Vertex2DColoredi32Buffer, camera: &Camera2D) {
        let pipeline = unsafe { &*(&self.render_pipeline_colored_i32 as *const _) };
        self.draw_custom(core, pipeline, vertex_data.as_u8(), camera, 0..1);
    }

    pub fn draw_textured_i32(&mut self, core: &Renderer, vertex_data: &Vertex2DTexturedi32Buffer, camera: &Camera2D) {
        let pipeline = unsafe { &*(&self.render_pipeline_textured_i32 as *const _) };
        self.draw_custom(core, pipeline, vertex_data.as_u8(), camera, 0..1);
    }

    pub fn draw_textured_i32_compute(&mut self, core: &Renderer, compute_vertex_data: &Vertex2DTexturedi32ComputeBuffer, camera: &Camera2D) {
        let compute_pipeline = unsafe { &*(&self.compute_pipeline_vertex_i32 as *const _) };
        self.draw_textured_i32_compute_custom(core,
            compute_pipeline,
            compute_vertex_data.as_u8(),
            camera
        );
    }

    pub fn draw_textured_i32_compute_custom(&mut self, core: &Renderer, compute_pipeline: &dyn ComputePreVertexPipeline2D, compute_data: &[u8], camera: &Camera2D) {
        let renderer_pipeline = unsafe { &*(&self.render_pipeline_textured_i32 as *const _) };
        self.draw_compute_custom(core, compute_pipeline, compute_data, renderer_pipeline, camera, 0..1);
    }

    pub fn draw_custom(&mut self, core: &Renderer, pipeline: &dyn RendererPipeline2D, vertex_data: &[u8], camera: &Camera2D, instances: std::ops::Range<u32>) {
        self.update_swapchain(core);
        self.update_uniform_buffer(core, camera);

        let max_buffer_size = self.vertex_buffer_size.min(self.mapped_buffer_size) as usize;
        let size_chunk = max_chunk(max_buffer_size, pipeline.vertex_size(), 3);
        
        let vertex_buffers = vertex_data.chunks(size_chunk);
        for vertex in vertex_buffers {
            self.update_vertex_buffer(core, vertex);

            self.vertex_buffer_actual_frame_size.vertex_count = vertex.len() / pipeline.vertex_size();
            self.vertex_buffer_actual_frame_size.vertex_size = vertex.len();

            if let Some(ssaa) = self.ssaa.as_mut() {
                ssaa.update_swapchain(core);
                self.submit_ssaa(core, pipeline.bind_group(), pipeline.pipeline(), instances.clone());
            } else {
                self.submit(core, pipeline.bind_group(), pipeline.pipeline(), instances.clone());
            }
        }
        
        if let Some(ssaa) = self.ssaa.as_mut() {
            ssaa.update_swapchain(core);
            ssaa.draw(core);
        }
    }

    pub fn draw_compute_custom(&mut self, core: &Renderer, compute_pipeline: &dyn ComputePreVertexPipeline2D, compute_data: &[u8], renderer_pipeline: &dyn RendererPipeline2D, camera: &Camera2D, instances: std::ops::Range<u32>) {
        self.update_swapchain(core);
        self.update_uniform_buffer(core, camera);
        
        let input_buffer_max = self.mapped_buffer_size.min(compute_pipeline.buffer_input_size()) as usize;
        let output_buffer_max = self.vertex_buffer_size.min(compute_pipeline.buffer_output_size()) as usize;

        let size_chunk = max_chunk_2(
            input_buffer_max, compute_pipeline.size_data() , 1,
            output_buffer_max, renderer_pipeline.vertex_size() * 6
        );

        
        let compute_vertex_buffers = compute_data.chunks(size_chunk);
        for compute_vertex in compute_vertex_buffers {
            core.device.poll(wgpu::Maintain::Wait);
            let mapped_buffer_slice = self.mapped_buffer.slice(0..compute_vertex.len() as u64);
            let vertex_data_map_write_future = mapped_buffer_slice.map_async(wgpu::MapMode::Write);
            core.device.poll(wgpu::Maintain::Poll);
            
            let _vertex_data_map_write = futures_executor::block_on(vertex_data_map_write_future).unwrap();
            mapped_buffer_slice.get_mapped_range_mut().copy_from_slice(compute_vertex);
            self.mapped_buffer.unmap();

            //self.mapped_buffer_cpu_access.as_mut().unwrap().as_slice().split_at_mut(compute_vertex.len()).0.copy_from_slice(compute_vertex);

            let mut command_encoder_mapped_buffer_copy = core.device.create_command_encoder(&wgpu::CommandEncoderDescriptor{ label: None });
            command_encoder_mapped_buffer_copy.copy_buffer_to_buffer(&self.mapped_buffer, 0, compute_pipeline.buffer_input(), 0, compute_vertex.len() as u64);
            core.queue.submit(vec![command_encoder_mapped_buffer_copy.finish()]);

            
            let mut command_encoder_compute_pass = core.device.create_command_encoder(&wgpu::CommandEncoderDescriptor{ label: None });
            {
                let mut compute_pass = command_encoder_compute_pass.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
                compute_pass.set_pipeline(compute_pipeline.pipeline());
                compute_pass.set_bind_group(0, compute_pipeline.bind_group(), &[]);
                compute_pass.dispatch(compute_vertex.len() as u32 / compute_pipeline.size_data() as u32, 1, 1);
            }
            core.queue.submit(vec![command_encoder_compute_pass.finish()]);
            core.device.poll(wgpu::Maintain::Poll);

            let mut command_encoder_output_buffer_copy = core.device.create_command_encoder(&wgpu::CommandEncoderDescriptor{ label: None });
            command_encoder_output_buffer_copy.copy_buffer_to_buffer(compute_pipeline.buffer_output(), 0, &self.vertex_buffer, 0, ((compute_vertex.len() / compute_pipeline.size_data()) * 6 * renderer_pipeline.vertex_size()) as u64);


            core.queue.submit(vec![command_encoder_output_buffer_copy.finish()]); //TODO: Regroup command encoders in one submit?
            core.device.poll(wgpu::Maintain::Poll);

            self.vertex_buffer_actual_frame_size.vertex_count = (compute_vertex.len() / compute_pipeline.size_data()) * 6;
            self.vertex_buffer_actual_frame_size.vertex_size = (compute_vertex.len() / compute_pipeline.size_data()) * 6 * renderer_pipeline.vertex_size();


            if let Some(ssaa) = self.ssaa.as_mut() {
                ssaa.update_swapchain(core);
                self.submit_ssaa(core, renderer_pipeline.bind_group(), renderer_pipeline.pipeline(), instances.clone());
            } else {
                self.submit(core, renderer_pipeline.bind_group(), renderer_pipeline.pipeline(), instances.clone());
            }
        }

        if let Some(ssaa) = self.ssaa.as_mut() {
            ssaa.update_swapchain(core);
            ssaa.draw(core);
        }
    }

    pub fn update_vertex_buffer(&mut self, core: &Renderer, vertex_data_raw: &[u8]) {
        if vertex_data_raw.len() == 0 { return }
        core.device.poll(wgpu::Maintain::Wait);

        let mapped_buffer_slice = self.mapped_buffer.slice(0..vertex_data_raw.len() as u64);
        let vertex_data_map_write_future = mapped_buffer_slice.map_async(wgpu::MapMode::Write);
        core.device.poll(wgpu::Maintain::Poll);

        let _vertex_data_map_write = futures_executor::block_on(vertex_data_map_write_future).unwrap();
        mapped_buffer_slice.get_mapped_range_mut().copy_from_slice(vertex_data_raw);
        self.mapped_buffer.unmap();

        let mut command_encoder = core.device.create_command_encoder(&wgpu::CommandEncoderDescriptor{label: None});
        command_encoder.copy_buffer_to_buffer(&self.mapped_buffer, 0, &self.vertex_buffer, 0, vertex_data_raw.len() as u64);
        core.queue.submit(vec!(command_encoder.finish()));
    }

    pub fn clear(&mut self, core: &Renderer, clear_info: ClearInfo) {
        self.update_swapchain(core);
        if self.ssaa.is_some() {
            self.clear_ssaa(core, clear_info.clone());
        }

        let (depth_buffer_load_op, frame_load_op) = match clear_info {
            ClearInfo::All(color) => { (wgpu::LoadOp::Clear(0.0), wgpu::LoadOp::Clear(color)) },
            ClearInfo::Frame(color) => { (wgpu::LoadOp::Load, wgpu::LoadOp::Clear(color)) },
            ClearInfo::Depth => { (wgpu::LoadOp::Clear(0.0), wgpu::LoadOp::Load ) }
        };

        let image_view = &core.actual_frame.frame.as_ref().unwrap().output.view;
        let mut command_encoder = core.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let depth_buffer_texture_view = self.depth_buffer_texture.create_view(&wgpu::TextureViewDescriptor {
                label: None,
                format: Some(wgpu::TextureFormat::Depth24Plus),
                dimension: Some(wgpu::TextureViewDimension::D2),
                aspect: wgpu::TextureAspect::DepthOnly,
                base_mip_level: 0,
                level_count: std::num::NonZeroU32::new(1),
                base_array_layer: 0,
                array_layer_count: std::num::NonZeroU32::new(1),
            });
            let depth_buffer = wgpu::RenderPassDepthStencilAttachmentDescriptor {
                attachment: &depth_buffer_texture_view,
                depth_ops: Some(wgpu::Operations {
                    load: depth_buffer_load_op,
                    store: true,
                }),
                stencil_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(0),
                    store: true,
                })
            };
            
            let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &image_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: frame_load_op,
                        store: true,
                    }
                }],
                depth_stencil_attachment: Some(depth_buffer),
            });
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(0..0));
            render_pass.set_index_buffer(self.vertex_buffer.slice(0..0), wgpu::IndexFormat::Uint32);
            render_pass.set_bind_group(0, self.render_pipeline_colored_i32.bind_group(), &[]);
            render_pass.set_pipeline(&self.render_pipeline_colored_i32.pipeline());
            render_pass.draw(0..0, 0..1);
        }
        core.queue.submit(vec![command_encoder.finish()]);
    }

    fn clear_ssaa(&mut self, core: &Renderer, clear_info: ClearInfo) {
        let ssaa = self.ssaa.as_mut().unwrap();
        
        let (depth_buffer_load_op, frame_load_op) = match clear_info {
            ClearInfo::All(color) => { (wgpu::LoadOp::Clear(0.0), wgpu::LoadOp::Clear(color)) },
            ClearInfo::Frame(color) => { (wgpu::LoadOp::Load, wgpu::LoadOp::Clear(color)) },
            ClearInfo::Depth => { (wgpu::LoadOp::Clear(0.0), wgpu::LoadOp::Load ) }
        };

        let image_view = ssaa.texture_frame.create_view(&wgpu::TextureViewDescriptor {
            label: None,
            format: Some(wgpu::TextureFormat::Bgra8Unorm),
            dimension: Some(wgpu::TextureViewDimension::D2),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            level_count: std::num::NonZeroU32::new(1),
            base_array_layer: 0,
            array_layer_count: std::num::NonZeroU32::new(1),
        });
        let mut command_encoder = core.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let depth_buffer_texture_view = ssaa.depth_buffer_texture.create_view(&wgpu::TextureViewDescriptor {
                label: None,
                format: Some(wgpu::TextureFormat::Depth24Plus),
                dimension: Some(wgpu::TextureViewDimension::D2),
                aspect: wgpu::TextureAspect::DepthOnly,
                base_mip_level: 0,
                level_count: std::num::NonZeroU32::new(1),
                base_array_layer: 0,
                array_layer_count: std::num::NonZeroU32::new(1),
            });
            let depth_buffer = wgpu::RenderPassDepthStencilAttachmentDescriptor {
                attachment: &depth_buffer_texture_view,
                depth_ops: Some(wgpu::Operations {
                    load: depth_buffer_load_op,
                    store: true,
                }),
                stencil_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(0),
                    store: true,
                })
            };
            
            let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &image_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: frame_load_op,
                        store: true,
                    }
                }],
                depth_stencil_attachment: Some(depth_buffer),
            });
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(0..0));
            render_pass.set_index_buffer(self.vertex_buffer.slice(0..0), wgpu::IndexFormat::Uint32);
            render_pass.set_bind_group(0, &self.render_pipeline_colored_i32.bind_group(), &[]);
            render_pass.set_pipeline(&self.render_pipeline_colored_i32.pipeline());
            render_pass.draw(0..0, 0..1);
        }
        core.queue.submit(vec![command_encoder.finish()]);
        ssaa.clear(core, clear_info, &self.render_pipeline_colored_i32);
    }

    fn submit(&mut self, core: &Renderer, bind_group: &wgpu::BindGroup, pipeline: &wgpu::RenderPipeline, instances: std::ops::Range<u32>) {
        let image_view = &core.actual_frame.frame.as_ref().unwrap().output.view;
        let mut command_encoder = core.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let depth_buffer_texture_view = self.depth_buffer_texture.create_view(&wgpu::TextureViewDescriptor {
                label: None,
                format: Some(wgpu::TextureFormat::Depth24Plus),
                dimension: Some(wgpu::TextureViewDimension::D2),
                aspect: wgpu::TextureAspect::DepthOnly,
                base_mip_level: 0,
                level_count: std::num::NonZeroU32::new(1),
                base_array_layer: 0,
                array_layer_count: std::num::NonZeroU32::new(1),
            });
            let depth_buffer = wgpu::RenderPassDepthStencilAttachmentDescriptor {
                attachment: &depth_buffer_texture_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                }),
                stencil_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                })
            };
            
            let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: image_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    }
                }],
                depth_stencil_attachment: Some(depth_buffer),
            });
            render_pass.set_bind_group(0, bind_group, &[]);
            render_pass.set_pipeline(pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(0..self.vertex_buffer_actual_frame_size.vertex_size as u64));
            render_pass.set_index_buffer(self.vertex_buffer.slice(0..0), wgpu::IndexFormat::Uint32);
            render_pass.draw(0..self.vertex_buffer_actual_frame_size.vertex_count as u32, instances);
        }
        core.queue.submit(vec![command_encoder.finish()]);
    }

    fn submit_ssaa(&mut self, core: &Renderer, bind_group: &wgpu::BindGroup, pipeline: &wgpu::RenderPipeline, instances: std::ops::Range<u32>) {
        let ssaa = self.ssaa.as_mut().unwrap();
        let image_view = ssaa.texture_frame.create_view(&wgpu::TextureViewDescriptor {
            label: None,
            format: Some(wgpu::TextureFormat::Bgra8Unorm),
            dimension: Some(wgpu::TextureViewDimension::D2),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            level_count: std::num::NonZeroU32::new(1),
            base_array_layer: 0,
            array_layer_count: std::num::NonZeroU32::new(1),
        });
        let mut command_encoder = core.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let depth_buffer_texture_view = ssaa.depth_buffer_texture.create_view(&wgpu::TextureViewDescriptor {
                label: None,
                format: Some(wgpu::TextureFormat::Depth24Plus),
                dimension: Some(wgpu::TextureViewDimension::D2),
                aspect: wgpu::TextureAspect::DepthOnly,
                base_mip_level: 0,
                level_count: std::num::NonZeroU32::new(1),
                base_array_layer: 0,
                array_layer_count: std::num::NonZeroU32::new(1),
            });
            let depth_buffer = wgpu::RenderPassDepthStencilAttachmentDescriptor {
                attachment: &depth_buffer_texture_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                }),
                stencil_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                })
            };
            
            let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &image_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    }
                }],
                depth_stencil_attachment: Some(depth_buffer),
            });
            render_pass.set_bind_group(0, bind_group, &[]);
            render_pass.set_pipeline(pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(0..self.vertex_buffer_actual_frame_size.vertex_size as u64));
            render_pass.set_index_buffer(self.vertex_buffer.slice(0..0), wgpu::IndexFormat::Uint32);
            render_pass.draw(0..self.vertex_buffer_actual_frame_size.vertex_count as u32, instances);
        }
        core.queue.submit(vec![command_encoder.finish()]);
    }
}

fn max_chunk(size_buffer: usize, size_data: usize, minimum_data_count: usize) -> usize {
    let size_data_total = size_data * minimum_data_count;
    return (size_buffer / size_data_total) * size_data_total
}


fn max_chunk_2(
    size_buffer_input: usize, size_data_input: usize, minimum_data_count_input: usize,
    size_buffer_output: usize, size_data_output: usize
) -> usize {
    let size_data_total_input = size_data_input * minimum_data_count_input;


    let size_buffer_input_chunked = (size_buffer_input / size_data_total_input) * size_data_total_input;

    let size_buffer_input_limited_chunked = (size_buffer_output / size_data_output) * size_data_total_input;
    
    return size_buffer_input_chunked.min(size_buffer_input_limited_chunked);

}