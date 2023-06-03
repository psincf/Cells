use crate::Renderer;
use euclid::default::Size2D;

pub enum SSAAFactor {
    Disabled,
    X2,
    X4,
    X8,
}

pub struct SSAA { // TODO: SSAA in separate pipeline
    swap_chain_size: Size2D<u32>,
    ratio: u32,
    pub texture_frame: wgpu::Texture,
    pub texture_frame_view: wgpu::TextureView,
    pub depth_buffer_texture: wgpu::Texture,
    bind_group_layout: wgpu::BindGroupLayout,
    render_pipeline: wgpu::RenderPipeline,
    empty_buffer: wgpu::Buffer,
}

impl SSAA {
    pub fn new(core: &Renderer, ratio: u32) -> SSAA {
        let swap_chain_size = core.swap_chain_size;

        let size_texture = swap_chain_size * 2u32.pow(ratio - 1);

        let texture_frame = core.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width : size_texture.width,
                height: size_texture.height,
                depth: 1
            },
            mip_level_count: ratio,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8Unorm,
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT | wgpu::TextureUsage::SAMPLED,
        });

        let texture_frame_view = texture_frame.create_view(&wgpu::TextureViewDescriptor {
                label: None,
                format: Some(wgpu::TextureFormat::Bgra8Unorm),
                dimension: Some(wgpu::TextureViewDimension::D2),
                aspect: wgpu::TextureAspect::All,
                base_mip_level: 0,
                level_count: None,
                base_array_layer: 0,
                array_layer_count: None,
        });

        let depth_buffer_texture = core.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width : size_texture.width,
                height: size_texture.height,
                depth: 1
            },
            mip_level_count: ratio,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth24Plus,
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
        });

        let bind_group_layout = core.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type : wgpu::TextureSampleType::Float { filterable: false },
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Sampler {
                        filtering: false,
                        comparison: false,
                    },
                    count: None,
                },
            ]
        });

        
        
        let vertex_shader = core.device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::util::make_spirv(&include_bytes!("../../shaders/ssaa/compiled/vert.spv")[..]),
            flags: wgpu::ShaderFlags::empty(),
        });

        let fragment_shader = core.device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::util::make_spirv(&include_bytes!("../../shaders/ssaa/compiled/frag.spv")[..]),
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
                buffers: &[],
                module: &vertex_shader,
                entry_point: "main",
            },
            fragment: Some(wgpu::FragmentState {
                targets: &[
                    wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Bgra8Unorm,
                        alpha_blend: wgpu::BlendState {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::Zero,
                            operation: wgpu::BlendOperation::Add,
                        },
                        /*
                        color_blend: wgpu::BlendState {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        */
                        color_blend: wgpu::BlendState {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::Zero,
                            operation: wgpu::BlendOperation::Add,
                        },
                        write_mask: wgpu::ColorWrite::ALL,
                    }
                ],
                module: &fragment_shader,
                entry_point: "main",
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
                depth_compare: wgpu::CompareFunction::Greater,
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
                }
            }),

            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            }
        });
        
        let empty_buffer = core.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            mapped_at_creation: false,
            size: 0,
            usage: wgpu::BufferUsage::COPY_DST | wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::INDEX,
        });

        SSAA {
            swap_chain_size,
            ratio,
            depth_buffer_texture,
            texture_frame,
            texture_frame_view,
            bind_group_layout,
            render_pipeline,
            empty_buffer,
        }
    }

    pub fn get_ssaa_factor(&self) -> SSAAFactor {
        match self.ratio {
            2 => { SSAAFactor::X2 }
            3 => { SSAAFactor::X4 }
            4 => { SSAAFactor::X8 }
            _ => { panic!("bug SSAA") }
        }
    }

    pub fn update_swapchain(&mut self, core: &Renderer) {
        if self.swap_chain_size != core.swap_chain_size {
            self.swap_chain_size = core.swap_chain_size;

            let size_texture = self.swap_chain_size * 2u32.pow(self.ratio - 1);

            self.texture_frame = core.device.create_texture(&wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d {
                    width : size_texture.width,
                    height: size_texture.height,
                    depth: 1
                },
                mip_level_count: self.ratio,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Bgra8Unorm,
                usage: wgpu::TextureUsage::RENDER_ATTACHMENT | wgpu::TextureUsage::SAMPLED,
            });

            self.texture_frame_view = self.texture_frame.create_view(&wgpu::TextureViewDescriptor {
                label: None,
                format: Some(wgpu::TextureFormat::Bgra8Unorm),
                dimension: Some(wgpu::TextureViewDimension::D2),
                aspect: wgpu::TextureAspect::All,
                base_mip_level: 0,
                level_count: None,
                base_array_layer: 0,
                array_layer_count: None,
            });

            self.depth_buffer_texture = core.device.create_texture(&wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d {
                    width : size_texture.width,
                    height: size_texture.height,
                    depth: 1
                },
                mip_level_count: self.ratio,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth24Plus,
                usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            });
        }
    }

    fn first_pass(&self, core: &Renderer) {
        for i in 1..self.ratio {
            let texture_frame_rendered = self.texture_frame.create_view(&wgpu::TextureViewDescriptor {
                label: None,
                format: Some(wgpu::TextureFormat::Bgra8Unorm),
                dimension: Some(wgpu::TextureViewDimension::D2),
                aspect: wgpu::TextureAspect::All,
                base_mip_level: i - 1,
                level_count: std::num::NonZeroU32::new(1),
                base_array_layer: 0,
                array_layer_count: std::num::NonZeroU32::new(1),
            });

            let sampler = core.device.create_sampler(&wgpu::SamplerDescriptor {
                label: None,
                border_color: None,
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Linear,
                lod_min_clamp: 0.0,
                lod_max_clamp: 0.0,
                compare: None,
                anisotropy_clamp: None,
            });

            let bind_group = core.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&texture_frame_rendered),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
            });

            let texture_frame_to_render = self.texture_frame.create_view(&wgpu::TextureViewDescriptor {
                label: None,
                format: Some(wgpu::TextureFormat::Bgra8Unorm),
                dimension: Some(wgpu::TextureViewDimension::D2),
                aspect: wgpu::TextureAspect::All,
                base_mip_level: i,
                level_count: std::num::NonZeroU32::new(1),
                base_array_layer: 0,
                array_layer_count: std::num::NonZeroU32::new(1),
            });

            let depth_buffer_texture_view = self.depth_buffer_texture.create_view(&wgpu::TextureViewDescriptor {
                label: None,
                format: Some(wgpu::TextureFormat::Depth24Plus),
                dimension: Some(wgpu::TextureViewDimension::D2),
                aspect: wgpu::TextureAspect::DepthOnly,
                base_mip_level: i,
                level_count: std::num::NonZeroU32::new(1),
                base_array_layer: 0,
                array_layer_count: std::num::NonZeroU32::new(1),
            });

            let depth_buffer = wgpu::RenderPassDepthStencilAttachmentDescriptor {
                attachment: &depth_buffer_texture_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(0.0),
                    store: true,
                }),
                stencil_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(0),
                    store: true,
                })
            };

            let mut command_encoder = core.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
            {
                let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                        attachment: &texture_frame_to_render,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            //load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            load: wgpu::LoadOp::Load,
                            store: true,
                        },
                    }],
                    depth_stencil_attachment: Some(depth_buffer),
                });
                render_pass.set_vertex_buffer(0, self.empty_buffer.slice(0..0));
                render_pass.set_index_buffer(self.empty_buffer.slice(0..0), wgpu::IndexFormat::Uint32);
                render_pass.set_bind_group(0, &bind_group, &[]);
                render_pass.set_pipeline(&self.render_pipeline);
                render_pass.draw(0..6, 0..1);
            }
            core.queue.submit(vec![command_encoder.finish()]);
        }
    }

    fn last_pass(&self, core: &Renderer) {
        let texture_frame_rendered = self.texture_frame.create_view(&wgpu::TextureViewDescriptor {
            label: None,
            format: Some(wgpu::TextureFormat::Bgra8Unorm),
            dimension: Some(wgpu::TextureViewDimension::D2),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: self.ratio - 1,
            level_count: std::num::NonZeroU32::new(1),
            base_array_layer: 0,
            array_layer_count: std::num::NonZeroU32::new(1),
        });

        let sampler = core.device.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            border_color: None,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: 0.0,
            compare: None,
            anisotropy_clamp: None,
        });
        let depth_buffer_texture_view = self.depth_buffer_texture.create_view(&wgpu::TextureViewDescriptor {
            label: None,
            format: Some(wgpu::TextureFormat::Depth24Plus),
            dimension: Some(wgpu::TextureViewDimension::D2),
            aspect: wgpu::TextureAspect::DepthOnly,
            base_mip_level: self.ratio - 1,
            level_count: std::num::NonZeroU32::new(1),
            base_array_layer: 0,
            array_layer_count: std::num::NonZeroU32::new(1),
        });

        let depth_buffer = wgpu::RenderPassDepthStencilAttachmentDescriptor {
            attachment: &depth_buffer_texture_view,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(0.0),
                store: true,
            }),
            stencil_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(0),
                store: true,
            })
        };

        let bind_group = core.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_frame_rendered),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        let texture_frame_to_render = &core.get_actual_frame().unwrap().output.view;

        let mut command_encoder = core.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &texture_frame_to_render,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    }
                }],
                depth_stencil_attachment: Some(depth_buffer),
            });
            render_pass.set_vertex_buffer(0, self.empty_buffer.slice(0..0));
            render_pass.set_index_buffer(self.empty_buffer.slice(0..0), wgpu::IndexFormat::Uint32);
            render_pass.set_bind_group(0, &bind_group, &[]);
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.draw(0..6, 0..1);
        }
        core.queue.submit(vec![command_encoder.finish()]);
    }
    
    pub fn draw(&mut self, core: &Renderer) {
        self.update_swapchain(core);
        self.first_pass(core);
        self.last_pass(core);
    }

    pub fn clear(&mut self, core: &Renderer, clear_info: crate::pipelines::generic_pipeline::ClearInfo, clear_pipeline: &crate::pipelines::renderer_pipeline_colored_i32::RendererPipelineColoredi32) {
        use crate::pipelines::generic_pipeline::ClearInfo;
        use crate::pipelines::generic_pipeline::RendererPipeline2D;
        self.update_swapchain(core);

        let (depth_buffer_load_op, frame_load_op) = match clear_info {
            ClearInfo::All(color) => { (wgpu::LoadOp::Clear(0.0), wgpu::LoadOp::Clear(color)) },
            ClearInfo::Frame(color) => { (wgpu::LoadOp::Load, wgpu::LoadOp::Clear(color)) },
            ClearInfo::Depth => { (wgpu::LoadOp::Clear(0.0), wgpu::LoadOp::Load ) }
        };

        for i in 1..self.ratio {
            let texture_frame_to_render = self.texture_frame.create_view(&wgpu::TextureViewDescriptor {
                label: None,
                format: Some(wgpu::TextureFormat::Bgra8Unorm),
                dimension: Some(wgpu::TextureViewDimension::D2),
                aspect: wgpu::TextureAspect::All,
                base_mip_level: i,
                level_count: std::num::NonZeroU32::new(1),
                base_array_layer: 0,
                array_layer_count: std::num::NonZeroU32::new(1),
            });

            let depth_buffer_texture_view = self.depth_buffer_texture.create_view(&wgpu::TextureViewDescriptor {
                label: None,
                format: Some(wgpu::TextureFormat::Depth24Plus),
                dimension: Some(wgpu::TextureViewDimension::D2),
                aspect: wgpu::TextureAspect::DepthOnly,
                base_mip_level: i,
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

            let mut command_encoder = core.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
            {
                let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                        attachment: &texture_frame_to_render,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: frame_load_op,
                            store: true,
                        },
                    }],
                    depth_stencil_attachment: Some(depth_buffer),
                });
                render_pass.set_vertex_buffer(0, self.empty_buffer.slice(0..0));
                render_pass.set_index_buffer(self.empty_buffer.slice(0..0), wgpu::IndexFormat::Uint32);
                render_pass.set_bind_group(0, &clear_pipeline.bind_group(), &[]);
                render_pass.set_pipeline(&clear_pipeline.pipeline());
                render_pass.draw(0..0, 0..1);
            }
            core.queue.submit(vec![command_encoder.finish()]);
        }
    }
}
