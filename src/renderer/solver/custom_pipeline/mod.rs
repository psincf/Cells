use wgpu_renderer::pipelines::generic_pipeline::RendererPipeline2D;

pub struct UniformBuffer2 {
    pub renderer_mode: i32,
    pub step: i32,
}

pub struct RenderPipeline {
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    pub pipeline: wgpu::RenderPipeline,
    pub staging_buffer: wgpu::util::StagingBelt,
    mapped_buffer: wgpu::Buffer,
    buffer_data_size: wgpu::BufferAddress,
    pub buffer_data: wgpu::Buffer,
    pub uniform_buffer_2: wgpu::Buffer,
}

impl RenderPipeline {
    pub fn new(core: &wgpu_renderer::Renderer, pipeline: &wgpu_renderer::Pipeline2D) -> RenderPipeline {
        let staging_buffer = wgpu::util::StagingBelt::new(512);
        let buffer_data_size = 10_000_000;
        let buffer_data = core.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: buffer_data_size,
            usage: wgpu::BufferUsage::COPY_DST |wgpu::BufferUsage::STORAGE,
            mapped_at_creation: false,
        });
        
        let mapped_buffer = core.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: buffer_data_size,
            usage: wgpu::BufferUsage::COPY_SRC |wgpu::BufferUsage::MAP_WRITE,
            mapped_at_creation: false,
        });

        let uniform_buffer_2 = core.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 512,
            usage: wgpu::BufferUsage::COPY_DST |wgpu::BufferUsage::UNIFORM,
            mapped_at_creation: false,
        });
        let bind_group_layout = core.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },

                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },

                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage {
                            read_only: true,
                        },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ]
        });
        let bind_group = core.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: pipeline.uniform_buffer.as_entire_binding(),
                },

                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: uniform_buffer_2.as_entire_binding(),
                },

                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: buffer_data.as_entire_binding(),
                }
            ],
        });
        
        let vertex_shader = core.device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::util::make_spirv(&include_bytes!("compiled/vert.spv")[..]),
            flags: wgpu::ShaderFlags::empty(),
        });
        let fragment_shader = core.device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::util::make_spirv(&include_bytes!("compiled/frag.spv")[..]),
            flags: wgpu::ShaderFlags::empty(),
        });

        let render_pipeline_layout = core.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor{        
            label: None,        
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = core.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vertex_shader,
                entry_point: "main",
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: 8,
                        step_mode: wgpu::InputStepMode::Vertex,
                        attributes: &[
                            wgpu::VertexAttribute {
                                offset: 0,
                                format: wgpu::VertexFormat::Int2,
                                shader_location: 0,
                            },
                        ],
                    },
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &fragment_shader,
                entry_point: "main",
                targets: &[
                    wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Bgra8Unorm,
                        alpha_blend: wgpu::BlendState {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::One,
                            operation: wgpu::BlendOperation::Max,
                        },
                        color_blend: wgpu::BlendState {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        write_mask: wgpu::ColorWrite::ALL,
                    },
                ]
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: Some(wgpu::IndexFormat::Uint32),
                front_face: wgpu::FrontFace::default(),
                cull_mode: wgpu::CullMode::None,
                polygon_mode: wgpu::PolygonMode::Fill,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                bias: wgpu::DepthBiasState::default(),
                clamp_depth: false,
                format: wgpu::TextureFormat::Depth24Plus,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::GreaterEqual,
                stencil: wgpu::StencilState {
                    front: wgpu::StencilFaceState {
                        compare: wgpu::CompareFunction::Always,
                        fail_op: wgpu::StencilOperation::Keep,
                        depth_fail_op: wgpu::StencilOperation::Keep,
                        pass_op: wgpu::StencilOperation::Keep,
                    },
                    back: wgpu::StencilFaceState {
                        compare: wgpu::CompareFunction::Always,
                        fail_op: wgpu::StencilOperation::Keep,
                        depth_fail_op: wgpu::StencilOperation::Keep,
                        pass_op: wgpu::StencilOperation::Keep,
                    },
                    read_mask: 0,
                    write_mask: 0,
                },
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            }
        });

        RenderPipeline {
            bind_group_layout,
            bind_group,
            pipeline: render_pipeline,
            staging_buffer,
            mapped_buffer,
            buffer_data_size,
            buffer_data,
            uniform_buffer_2,
        }
    }

    pub fn get_original_vertex(&self) -> [i32;12] {
        [
            -1, -1,
            1, -1,
            -1, 1,
            -1, 1,
            1, -1,
            1, 1,
        ]
    }

    pub fn update_uniform_buffer_2(&mut self, core: &wgpu_renderer::Renderer, data: UniformBuffer2) {
        let data = [data];
        core.queue.write_buffer(&self.uniform_buffer_2, 0, unsafe { data.align_to().1 });
    }

    pub fn update_buffer_data(&mut self, core: &wgpu_renderer::Renderer, data: &[crate::game::entity::DrawableEntity]) {
        if data.len() == 0 { return }
        let mut commmand_encoder = core.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {label: None});
        {
            let slice = self.mapped_buffer.slice(..);
            let future = slice.map_async(wgpu::MapMode::Write);
            core.device.poll(wgpu::Maintain::Poll);
            futures_executor::block_on(future).unwrap();
            let mut buffer = slice.get_mapped_range_mut();
            unsafe { std::ptr::copy_nonoverlapping(&data[0], buffer.as_mut_ptr() as *mut _, data.len()); }
        }
        self.mapped_buffer.unmap();
        commmand_encoder.copy_buffer_to_buffer(&self.mapped_buffer, 0, &self.buffer_data, 0, (data.len() * std::mem::size_of::<crate::game::entity::DrawableEntity>()) as u64);

        core.queue.submit(vec![commmand_encoder.finish()]);
    }

    pub fn update_buffer_data_vec_chunk(&mut self, core: &wgpu_renderer::Renderer, data: &crate::utils::vec_chunk::VecChunk<crate::game::entity::DrawableEntity>) {
        if data.len() == 0 { return }
        let mut commmand_encoder = core.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {label: None});
        {
            let slice = self.mapped_buffer.slice(..);
            let future = slice.map_async(wgpu::MapMode::Write);
            core.device.poll(wgpu::Maintain::Poll);
            futures_executor::block_on(future).unwrap();
            let mut buffer = slice.get_mapped_range_mut();
            let buffer_ptr = buffer.as_mut_ptr();
            for (count, entity) in data.iter().enumerate() {
                unsafe { std::ptr::copy(entity, (buffer_ptr as *mut crate::game::entity::DrawableEntity).add(count), 1); }
            }
        }
        self.mapped_buffer.unmap();
        commmand_encoder.copy_buffer_to_buffer(&self.mapped_buffer, 0, &self.buffer_data, 0, (data.len() * std::mem::size_of::<crate::game::entity::DrawableEntity>()) as u64);

        core.queue.submit(vec![commmand_encoder.finish()]);
    }

    pub fn get_buffer_data_size(&mut self) -> u64 {
        self.buffer_data_size
    }

    fn update_bind_group(&mut self, core: &wgpu_renderer::Renderer, generic_pipeline: &wgpu_renderer::pipelines::generic_pipeline::Pipeline2D) {
        self.bind_group = core.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: generic_pipeline.uniform_buffer.as_entire_binding(),
                },

                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.uniform_buffer_2.as_entire_binding(),
                },

                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.buffer_data.as_entire_binding(),
                }
            ],
        });
    }

    pub fn set_buffer_data_size(&mut self, core: &wgpu_renderer::Renderer, generic_pipeline: &wgpu_renderer::pipelines::generic_pipeline::Pipeline2D, size: wgpu::BufferAddress) {
        if size == self.buffer_data_size { return }
        self.buffer_data_size = size;
        self.buffer_data = core.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: self.buffer_data_size,
            usage: wgpu::BufferUsage::COPY_DST |wgpu::BufferUsage::STORAGE,
            mapped_at_creation: false,
        });

        self.mapped_buffer = core.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: self.buffer_data_size,
            usage: wgpu::BufferUsage::COPY_SRC |wgpu::BufferUsage::MAP_WRITE,
            mapped_at_creation: false,
        });

        self.update_bind_group(core, generic_pipeline);
    }
}

impl RendererPipeline2D for RenderPipeline {
    fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    fn pipeline(&self) -> &wgpu::RenderPipeline {
        &self.pipeline
    }

    fn vertex_size(&self) -> usize {
        8
    }
}