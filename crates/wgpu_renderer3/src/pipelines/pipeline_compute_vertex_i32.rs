use crate::Renderer;
//use crate::pipelines::pipeline_2d::ComputePreVertexPipeline2D;
use crate::pipelines::generic_pipeline::ComputePreVertexPipeline2D;
use crate::vertex_data::Vertex2DTexturedi32Compute;

pub struct PipelineComputeVertexi32 {
    pub buffer_input: wgpu::Buffer,
    pub buffer_input_size: wgpu::BufferAddress,
    pub buffer_output: wgpu::Buffer,
    pub buffer_output_size: wgpu::BufferAddress,
    pub bind_group: wgpu::BindGroup,
    pub compute_pipeline: wgpu::ComputePipeline,
}

impl PipelineComputeVertexi32 {
    pub fn new(core: &Renderer) -> PipelineComputeVertexi32 {
        let device = &core.device;
        
        let shader_pre_render = core.device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::util::make_spirv(&include_bytes!("../../shaders/2D_textured/compiled/comp.spv")[..]),
            flags: wgpu::ShaderFlags::empty(),
        });
        
        let buffer_input_size = 2_000_000;
        let buffer_input = device.create_buffer(&wgpu::BufferDescriptor {           
            label: None, 
            mapped_at_creation: false,
            size: buffer_input_size,
            usage: wgpu::BufferUsage::COPY_DST | wgpu::BufferUsage::STORAGE,
        });

        let buffer_output_size = 10_000_000;
        let buffer_output = device.create_buffer(&wgpu::BufferDescriptor {            
            label: None,
            mapped_at_creation: false,
            size: buffer_output_size,
            usage: wgpu::BufferUsage::COPY_SRC | wgpu::BufferUsage::STORAGE,
        });
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage {
                            read_only: false,
                        },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStage::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage {
                            read_only: false,
                        },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ]
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {   
            label: None,         
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer_input.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: buffer_output.as_entire_binding(),
                },
            ],
        });

        let compute_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&compute_pipeline_layout),                       
            module: &shader_pre_render,
            entry_point: "main",
        });
        
        PipelineComputeVertexi32 {
            buffer_input,
            buffer_input_size,
            buffer_output,
            buffer_output_size,
            bind_group,
            compute_pipeline,
        }
    }
}

impl ComputePreVertexPipeline2D for PipelineComputeVertexi32 {
    fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
    fn buffer_input(&self) -> &wgpu::Buffer {
        &self.buffer_input
    }
    fn buffer_input_size(&self) -> u64 {
        self.buffer_input_size
    }
    fn buffer_output(&self) -> &wgpu::Buffer {
        &self.buffer_output
    }
    fn buffer_output_size(&self) -> u64 {
        self.buffer_output_size
    }
    fn pipeline(&self) -> &wgpu::ComputePipeline {
        &self.compute_pipeline
    }
    fn size_data(&self) -> usize {
        std::mem::size_of::<Vertex2DTexturedi32Compute>() * 1
    }
}