#[cfg(feature = "wgpu_6")]
use wgpu_v6 as wgpu;

#[cfg(feature = "wgpu_7")]
use wgpu_v7 as wgpu;

#[cfg(feature = "wgpu_6")]
pub struct EguiRendererWgpu {
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: Option<wgpu::BindGroup>,
    uniform_buffer: wgpu::Buffer,
    mapped_buffer_size: u64,
    mapped_buffer: wgpu::Buffer,
    vertex_buffer_size: u64,
    vertex_buffer: wgpu::Buffer,
    texture: Option<wgpu::Texture>,
    render_pipeline_layout: wgpu::PipelineLayout,
    render_pipeline: wgpu::RenderPipeline,
}

#[cfg(feature = "wgpu_6")]
impl EguiRendererWgpu {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> EguiRendererWgpu {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::UniformBuffer {
                        dynamic: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::SampledTexture {
                        dimension: wgpu::TextureViewDimension::D2,
                        component_type: wgpu::TextureComponentType::Float,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Sampler {
                        comparison: false,
                    },
                    count: None,
                },
            ]
        });
        
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 1_024,
            usage: wgpu::BufferUsage::COPY_DST | wgpu::BufferUsage::UNIFORM,
            mapped_at_creation: false,
        });

        let mapped_buffer_size = 1_000_000;
        let mapped_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: mapped_buffer_size,
            usage: wgpu::BufferUsage::COPY_SRC | wgpu::BufferUsage::MAP_WRITE,
            mapped_at_creation: false,
        });

        let vertex_buffer_size = 1_000_000;
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: vertex_buffer_size,
            usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
            mapped_at_creation: false,
        });

        let vertex_shader_textured_i32 = device.create_shader_module(wgpu::util::make_spirv(&include_bytes!("../shaders/compiled/vert.spv")[..]));
        let fragment_shader_textured = device.create_shader_module(wgpu::util::make_spirv(&include_bytes!("../shaders/compiled/frag.spv")[..]));

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor{
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[]
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&render_pipeline_layout),
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vertex_shader_textured_i32,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &fragment_shader_textured,
                entry_point: "main",
            }),
            rasterization_state: None,
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: &[
                wgpu::ColorStateDescriptor {
                    format: wgpu::TextureFormat::Bgra8Unorm,
                    alpha_blend: wgpu::BlendDescriptor {
                        src_factor: wgpu::BlendFactor::One,
                        dst_factor: wgpu::BlendFactor::Zero,
                        operation: wgpu::BlendOperation::Add,
                    },
                    color_blend: wgpu::BlendDescriptor {
                        src_factor: wgpu::BlendFactor::One,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                    write_mask: wgpu::ColorWrite::ALL,
                }
            ],
            depth_stencil_state: None /* Some(wgpu::DepthStencilStateDescriptor {
                format: wgpu::TextureFormat::Depth24Plus,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Greater,
                stencil: wgpu::StencilStateDescriptor {
                    front: wgpu::StencilStateFaceDescriptor {
                        compare: wgpu::CompareFunction::Always,
                        fail_op: wgpu::StencilOperation::Keep,
                        depth_fail_op: wgpu::StencilOperation::Keep,
                        pass_op: wgpu::StencilOperation::Keep,
                    },
                    back: wgpu::StencilStateFaceDescriptor {
                        compare: wgpu::CompareFunction::Always,
                        fail_op: wgpu::StencilOperation::Keep,
                        depth_fail_op: wgpu::StencilOperation::Keep,
                        pass_op: wgpu::StencilOperation::Keep,
                    },
                    read_mask: 0,
                    write_mask: 0,
                },
            })*/,
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint32,
                vertex_buffers: &[
                    wgpu::VertexBufferDescriptor {
                        stride: 20,
                        step_mode: wgpu::InputStepMode::Vertex,
                        attributes: &[
                            wgpu::VertexAttributeDescriptor {
                                offset: 0,
                                format: wgpu::VertexFormat::Float2,
                                shader_location: 0,
                            },
                            wgpu::VertexAttributeDescriptor {
                                offset: 8,
                                format: wgpu::VertexFormat::Float2,
                                shader_location: 1,
                            },
                            wgpu::VertexAttributeDescriptor {
                                offset: 16,
                                format: wgpu::VertexFormat::Float,
                                shader_location: 2,
                            },
                        ],
                    },
                ],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        EguiRendererWgpu {
            bind_group_layout: bind_group_layout,
            bind_group: None,
            uniform_buffer,
            mapped_buffer_size,
            mapped_buffer,
            vertex_buffer_size,
            vertex_buffer,
            texture: None,
            render_pipeline_layout,
            render_pipeline,
        }

    }
    pub fn upload_texture(&mut self, egui_context: &egui::Context, device: &wgpu::Device, queue: &wgpu::Queue) {
        let egui_texture = egui_context.texture();
    
        self.texture = Some(
            device.create_texture(&wgpu::TextureDescriptor {  
                label: None,
                size: wgpu::Extent3d {
                    width: egui_texture.width as u32,
                    height: egui_texture.height as u32,
                    depth: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsage::COPY_DST | wgpu::TextureUsage::SAMPLED,
            })
        );

        let texture_view = self.texture.as_mut().unwrap().create_view(&wgpu::TextureViewDescriptor {
            label: None,
            format: Some(wgpu::TextureFormat::Rgba8Unorm),
            dimension: Some(wgpu::TextureViewDimension::D2),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        });

        let sampler = device.create_sampler(
            &wgpu::SamplerDescriptor {
                label: None,
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Nearest,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::FilterMode::Nearest,
                lod_min_clamp: 0.0,
                lod_max_clamp: 100.0,
                compare: None,
                anisotropy_clamp: None
            }
        );

        let pixels: Vec<[u8;4]> = egui_texture.srgba_pixels().map(|t| t.to_array()).collect();
        
        let texture_copy = wgpu::TextureCopyView {
            texture: self.texture.as_ref().unwrap(),
            mip_level: 0,
            origin: wgpu::Origin3d {
                x: 0,
                y: 0,
                z: 0,
            }
        };
        queue.write_texture(texture_copy, &unsafe { &pixels.align_to().1 }, wgpu::TextureDataLayout {
            offset: 0,
            bytes_per_row: (egui_texture.width * 4) as u32,
            rows_per_image: egui_texture.height as u32,
        }, wgpu::Extent3d { width: egui_texture.width as u32, height: egui_texture.height as u32, depth: 1 });

        self.bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(self.uniform_buffer.slice(..)),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        }));

    }

    pub fn draw_egui(&mut self, paint_jobs: Vec<(egui::Rect, egui::paint::Triangles)>, device: &wgpu::Device, queue: &wgpu::Queue, frame: &wgpu::TextureView, size_window: [f32;2]) {
        assert!( self.bind_group.is_some() );
        for (mut rect, triangles) in paint_jobs.iter() {
            rect.max.x = rect.max.x.min(std::f32::MAX);
            rect.max.y = rect.max.y.min(std::f32::MAX);
            let mut vertex = Vec::new();
            for &i in triangles.indices.iter() {
                vertex.push(triangles.vertices[i as usize]);
            }
            queue.write_buffer(&self.uniform_buffer, 0, unsafe { [(size_window, rect)].align_to().1 });
            queue.write_buffer(&self.vertex_buffer, 0, unsafe { vertex.align_to().1 });
            /*
            {
                let mapped_buffer = self.mapped_buffer.slice(0..vertex.len() as u64 * std::mem::size_of::<egui::paint::Vertex>() as u64);
                mapped_buffer.map_async(wgpu::MapMode::Write);
                device.poll(wgpu::Maintain::Wait);
                let mut buffer_view = mapped_buffer.get_mapped_range_mut();
                buffer_view.copy_from_slice(unsafe { vertex.align_to().1 });
            }
    
            self.mapped_buffer.unmap();
            */

            let mut command_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

            {
                let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                        attachment: frame,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: true,
                        },
                    }],
                    depth_stencil_attachment: None,
                });

                render_pass.set_scissor_rect(rect.min.x as u32, rect.min.y as u32, rect.width() as u32, rect.height() as u32);
                render_pass.set_bind_group(0, self.bind_group.as_ref().unwrap(), &[]);
                render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(0..vertex.len() as u64 * std::mem::size_of::<egui::paint::Vertex>() as u64));
                render_pass.set_pipeline(&self.render_pipeline);
                render_pass.draw(0..vertex.len() as u32, 0..1);
            }
            
            queue.submit(vec![command_encoder.finish()]);
        }
    }
}

#[cfg(feature = "wgpu_7")]
#[allow(dead_code)]
pub struct EguiRendererWgpu {
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: Option<wgpu::BindGroup>,
    uniform_buffer: wgpu::Buffer,
    //depth_buffer: wgpu::Texture,
    mapped_buffer_size: u64,
    mapped_buffer: wgpu::Buffer,
    vertex_buffer_size: u64,
    vertex_buffer: wgpu::Buffer,
    texture: Option<wgpu::Texture>,
    render_pipeline_layout: wgpu::PipelineLayout,
    render_pipeline: wgpu::RenderPipeline,
}

#[cfg(feature = "wgpu_7")]
impl EguiRendererWgpu {
    pub fn new(device: &wgpu::Device, _queue: &wgpu::Queue, _size_window: [f32;2]) -> EguiRendererWgpu {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                    ty: wgpu::BindingType::Texture {
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Sampler {
                        comparison: false,
                        filtering: false,
                    },
                    count: None,
                },
            ]
        });
        
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 1_024,
            usage: wgpu::BufferUsage::COPY_DST | wgpu::BufferUsage::UNIFORM,
            mapped_at_creation: false,
        });
        /*
        let depth_buffer = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: size_window[0] as u32,
                height: size_window[1] as u32,
                depth: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth24Plus,
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
        });
        */
        let mapped_buffer_size = 1_000_000;
        let mapped_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: mapped_buffer_size,
            usage: wgpu::BufferUsage::COPY_SRC | wgpu::BufferUsage::MAP_WRITE,
            mapped_at_creation: false,
        });

        let vertex_buffer_size = 1_000_000;
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: vertex_buffer_size,
            usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST | wgpu::BufferUsage::INDEX,
            mapped_at_creation: false,
        });
        
        let vertex_shader_textured_i32 = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::util::make_spirv(&include_bytes!("../shaders/compiled/vert.spv")[..]),
            flags: wgpu::ShaderFlags::empty(),
        });

        let fragment_shader_textured = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::util::make_spirv(&include_bytes!("../shaders/compiled/frag.spv")[..]),
            flags: wgpu::ShaderFlags::empty(),
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[]
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vertex_shader_textured_i32,
                entry_point: "main",
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: 20,
                        step_mode: wgpu::InputStepMode::Vertex,
                        attributes: &[
                            wgpu::VertexAttribute {
                                offset: 0,
                                format: wgpu::VertexFormat::Float2,
                                shader_location: 0,
                            },
                            wgpu::VertexAttribute {
                                offset: 8,
                                format: wgpu::VertexFormat::Float2,
                                shader_location: 1,
                            },
                            wgpu::VertexAttribute {
                                offset: 16,
                                format: wgpu::VertexFormat::Float,
                                shader_location: 2,
                            },
                        ],
                    },
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &fragment_shader_textured,
                entry_point: "main",
                targets: &[
                    wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Bgra8Unorm,
                        alpha_blend: wgpu::BlendState {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::Zero,
                            operation: wgpu::BlendOperation::Add,
                        },
                        color_blend: wgpu::BlendState {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        write_mask: wgpu::ColorWrite::ALL,
                    }
                ]
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: Some(wgpu::IndexFormat::Uint32),
                front_face: wgpu::FrontFace::default(),
                cull_mode: wgpu::CullMode::None,
                polygon_mode: wgpu::PolygonMode::Fill,
            },
            depth_stencil: None, /*Some(wgpu::DepthStencilState {
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
                },
            }),*/
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            }
        });

        EguiRendererWgpu {
            bind_group_layout: bind_group_layout,
            bind_group: None,
            uniform_buffer,
            //depth_buffer,
            mapped_buffer_size,
            mapped_buffer,
            vertex_buffer_size,
            vertex_buffer,
            texture: None,
            render_pipeline_layout,
            render_pipeline,
        }

    }
    pub fn upload_texture(&mut self, egui_context: &egui::Context, device: &wgpu::Device, queue: &wgpu::Queue) {
        let egui_texture = egui_context.texture();
    
        self.texture = Some(
            device.create_texture(&wgpu::TextureDescriptor {  
                label: None,
                size: wgpu::Extent3d {
                    width: egui_texture.width as u32,
                    height: egui_texture.height as u32,
                    depth: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsage::COPY_DST | wgpu::TextureUsage::SAMPLED,
            })
        );

        let texture_view = self.texture.as_mut().unwrap().create_view(&wgpu::TextureViewDescriptor {
            label: None,
            format: Some(wgpu::TextureFormat::Rgba8Unorm),
            dimension: Some(wgpu::TextureViewDimension::D2),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        });

        let sampler = device.create_sampler(
            &wgpu::SamplerDescriptor {
                label: None,
                border_color: None,
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Nearest,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::FilterMode::Nearest,
                lod_min_clamp: 0.0,
                lod_max_clamp: 100.0,
                compare: None,
                anisotropy_clamp: None
            }
        );

        let pixels: Vec<[u8;4]> = egui_texture.srgba_pixels().map(|t| t.to_array()).collect();
        
        let texture_copy = wgpu::TextureCopyView {
            texture: self.texture.as_ref().unwrap(),
            mip_level: 0,
            origin: wgpu::Origin3d {
                x: 0,
                y: 0,
                z: 0,
            }
        };
        queue.write_texture(texture_copy, &unsafe { &pixels.align_to().1 }, wgpu::TextureDataLayout {
            offset: 0,
            bytes_per_row: (egui_texture.width * 4) as u32,
            rows_per_image: egui_texture.height as u32,
        }, wgpu::Extent3d { width: egui_texture.width as u32, height: egui_texture.height as u32, depth: 1 });

        self.bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        }));

    }

    fn update_size_vertex_buffer(&mut self, device: &wgpu::Device, new_size: u64) {
        self.vertex_buffer_size = new_size;
        
        self.vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: self.vertex_buffer_size,
            usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST | wgpu::BufferUsage::INDEX,
            mapped_at_creation: false,
        });
    }

    pub fn draw_egui(&mut self, paint_jobs: Vec<egui::epaint::ClippedMesh>, device: &wgpu::Device, queue: &wgpu::Queue, frame: &wgpu::TextureView, size_window: [f32;2]) {
        assert!( self.bind_group.is_some() );
        for paint_job in paint_jobs.iter() {
            let egui::paint::ClippedMesh(mut rect, triangles) = paint_job;
            rect.min.x = rect.min.x.max(0.0);
            rect.min.y = rect.min.y.max(0.0);

            if rect.min.x >= size_window[0] { continue }
            if rect.min.y >= size_window[1] { continue }

            rect.max.x = rect.max.x.min(std::f32::MAX).min(size_window[0]);
            rect.max.y = rect.max.y.min(std::f32::MAX).min(size_window[1]);

            if rect.width() as u32 <= 0 { continue }
            if rect.height() as u32 <= 0 { continue }

            if rect.min.x >= rect.max.x { continue }
            if rect.min.y >= rect.max.y { continue }
            
            let mut vertex = Vec::new();
            for &i in triangles.indices.iter() {
                vertex.push(triangles.vertices[i as usize]);
            }

            while (vertex.len() * std::mem::size_of::<egui::paint::Vertex>()) as u64 > self.vertex_buffer_size {
                self.update_size_vertex_buffer(device, self.vertex_buffer_size * 2);
            }


            queue.write_buffer(&self.uniform_buffer, 0, unsafe { [(size_window, rect)].align_to().1 });
            queue.write_buffer(&self.vertex_buffer, 0, unsafe { vertex.align_to().1 });

            /*
            {
                let mapped_buffer = self.mapped_buffer.slice(0..vertex.len() as u64 * std::mem::size_of::<egui::paint::Vertex>() as u64);
                mapped_buffer.map_async(wgpu::MapMode::Write);
                device.poll(wgpu::Maintain::Wait);
                let mut buffer_view = mapped_buffer.get_mapped_range_mut();
                dbg!(vertex.len());
                buffer_view.copy_from_slice(unsafe { vertex.align_to().1 });
            }
    
            self.mapped_buffer.unmap();
            */
            
            /*
            let depth_view = self.depth_buffer.create_view(&wgpu::TextureViewDescriptor {
                label: None,
                format: Some(wgpu::TextureFormat::Depth24Plus),
                dimension: Some(wgpu::TextureViewDimension::D2),
                aspect: wgpu::TextureAspect::DepthOnly,
                base_mip_level: 0,
                level_count: None,
                base_array_layer: 0,
                array_layer_count: None,
            });
            let depth = wgpu::RenderPassDepthStencilAttachmentDescriptor {
                attachment: &depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(0.0),
                    store: true,
                }),
                stencil_ops: None,
            };
            */

            let mut command_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

            {
                let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                        attachment: frame,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: true,
                        },
                    }],
                    depth_stencil_attachment: None,
                });

                render_pass.set_scissor_rect(rect.min.x as u32, rect.min.y as u32, rect.width() as u32, rect.height() as u32);
                render_pass.set_bind_group(0, self.bind_group.as_ref().unwrap(), &[]);
                render_pass.set_index_buffer(self.vertex_buffer.slice(0..0), wgpu::IndexFormat::Uint32);
                render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(0..vertex.len() as u64 * std::mem::size_of::<egui::paint::Vertex>() as u64));
                render_pass.set_pipeline(&self.render_pipeline);
                render_pass.draw(0..vertex.len() as u32, 0..1);
            }
            
            queue.submit(vec![command_encoder.finish()]);
        }
    }
}