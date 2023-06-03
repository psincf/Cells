use crate::Renderer;
use crate::pipelines::generic_pipeline::RendererPipeline2D;
use crate::vertex_data::Vertex2DTexturedi32;

use std::collections::HashMap;
use euclid::default::{Point2D, Rect, Size2D};

#[derive(Clone, Copy)]
pub struct Texture {
    pub layer: i32,
    pub rect: Rect<i32>,
}

pub struct RendererTextureInfo {
    pub list: Vec<Texture>,
    pub index_hashmap: HashMap<String, usize>,
    pub max_layer: u32,
    pub size: u32,
}
pub struct RendererPipelineTexturedi32 {
    bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
    pub textures: Vec<wgpu::Texture>,
    pub textures_info: RendererTextureInfo,
}

impl RendererPipelineTexturedi32 {
    pub fn new(core: &Renderer, uniform_buffer: &wgpu::Buffer) -> RendererPipelineTexturedi32 {
        let textures_info = RendererTextureInfo {
            list: Vec::new(),
            index_hashmap: HashMap::new(),
            max_layer: 1,
            size: 4096,
        };
        let texture = core.device.create_texture(&wgpu::TextureDescriptor {  
            label: None,
            size: wgpu::Extent3d {
                width: textures_info.size,
                height: textures_info.size,
                depth: textures_info.max_layer,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsage::COPY_DST | wgpu::TextureUsage::SAMPLED,
        });
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: None,
            format: Some(wgpu::TextureFormat::Rgba8Unorm),
            dimension: Some(wgpu::TextureViewDimension::D2),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            level_count: std::num::NonZeroU32::new(1),
            base_array_layer: 0,
            array_layer_count: std::num::NonZeroU32::new(textures_info.max_layer),
        });
        let mut textures = Vec::new();
        textures.push(texture);

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
                    ty: wgpu::BindingType::Texture {
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float {
                            filterable: false,
                        },
                        multisampled: false,
                    },
                    count: std::num::NonZeroU32::new(0),
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Sampler {
                        filtering: false,
                        comparison: false,
                    },
                    count: None,
                },
            ]
        });
        let bind_group = core.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
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
        });
        
        let vertex_shader_textured_i32 = core.device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::util::make_spirv(&include_bytes!("../../shaders/2D_textured/compiled/vert_i32.spv")[..]),
            flags: wgpu::ShaderFlags::empty(),
        });

        let fragment_shader_textured = core.device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::util::make_spirv(&include_bytes!("../../shaders/2D_textured/compiled/frag.spv")[..]),
            flags: wgpu::ShaderFlags::empty(),
        });
        
        let render_pipeline_layout = core.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor{
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[]
        });
        let render_pipeline = core.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: 28,
                        step_mode: wgpu::InputStepMode::Vertex,
                        attributes: &[
                            wgpu::VertexAttribute {
                                offset: 0,
                                format: wgpu::VertexFormat::Float3,
                                shader_location: 0,
                            },
                            wgpu::VertexAttribute {
                                offset: 12,
                                format: wgpu::VertexFormat::Float3,
                                shader_location: 1,
                            },
                        ],
                    },
                ],
                module: &vertex_shader_textured_i32,
                entry_point: "main",
            },
            fragment: Some(wgpu::FragmentState {
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
                    }
                ],
                module: &fragment_shader_textured,
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
                },
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            }
        });

        RendererPipelineTexturedi32 {
            bind_group,
            render_pipeline,
            textures,
            textures_info,
        }
    }

    fn create_megatexture(&mut self, textures: &[(String, Vec<u8>)], size: u32) -> Vec<image::RgbaImage> {
        use image::GenericImage;
        let gap = size; // TODO: change that
        
        // Create the images array and the first image
        let mut images: Vec<image::RgbaImage> = Vec::new();
        images.push(image::ImageBuffer::new(self.textures_info.size, self.textures_info.size));

        // Iterate textures and update the array of images
        let mut x_offset = 0;
        let mut y_offset = 0;
        let mut layer = 0;
        for (name, texture) in textures.iter() {
            let image_cell = image::load_from_memory(texture).unwrap();
            images[layer as usize].copy_from(&image_cell, x_offset, y_offset).unwrap();
            self.textures_info.list.push(Texture {
                layer: layer,
                rect: Rect::new(
                    Point2D::new(x_offset as i32, y_offset as i32),
                    Size2D::new(gap as i32, gap as i32)
                ),
            });

            // Add the name and indice of the texture to Hashmap and check if the name is already taken
            let already_taken = self.textures_info.index_hashmap.insert(name.clone(), self.textures_info.list.len() - 1);
            if let Some(_already_taken) = already_taken { panic!("texture name already taken!") }

            // Update offset
            x_offset += gap;
            if x_offset + gap > self.textures_info.size {
                x_offset = 0;
                y_offset += gap;
                if y_offset + gap > self.textures_info.size {
                    y_offset = 0;
                    layer += 1;
                    if layer >= self.textures_info.max_layer as i32 {
                        panic!("megaTexture too big!")
                    }
                    images.push(image::ImageBuffer::new(self.textures_info.size, self.textures_info.size));
                }
            }
        }
        return images;
    }

    pub fn init_texture(&mut self, core: &Renderer, textures: &[(String, Vec<u8>)], size: u32) {
        let mut command_encoder = core.device.create_command_encoder(&wgpu::CommandEncoderDescriptor{ label: None } );
        let texture_size = self.textures_info.size;
        let images = self.create_megatexture(textures, size);

        for layer in 0..images.len() {
            if layer > 0 { panic!("Can't with multiple players") };
            let image = images[layer].clone().into_raw();

            let buffer = core.device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: image.len() as u64,
                usage: wgpu::BufferUsage::MAP_WRITE | wgpu::BufferUsage::COPY_SRC,
                mapped_at_creation: true,
            });
            let slice = buffer.slice(..);
            {
                let mut mapped_buffer = slice.get_mapped_range_mut();
                mapped_buffer.copy_from_slice(&image);
            }
            buffer.unmap();
            let buffer_copy_view = wgpu::BufferCopyView {
                buffer: &buffer,
                layout: wgpu::TextureDataLayout {
                    offset: 0,
                    bytes_per_row: texture_size * 4,
                    rows_per_image: texture_size,
                }
            };
            let texture_copy_view = wgpu::TextureCopyView {
                texture: &self.textures[0],
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: 0,
                    y: 0,
                    z: layer as u32,
                },
            };
            let extent3d = wgpu::Extent3d {            
                width: texture_size,
                height: texture_size,
                depth: 1,
            };
            command_encoder.copy_buffer_to_texture(buffer_copy_view, texture_copy_view, extent3d);

            // image::save_buffer(layer.to_string() + "aaa.png", &image, texture_size, texture_size, image::ColorType::Rgba8).unwrap();
            // dbg!(&self.textures_info.index_hashmap);
            
        }
        core.queue.submit(vec!(command_encoder.finish()));
    }

}

impl RendererPipeline2D for RendererPipelineTexturedi32 {
    fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
    fn pipeline(&self) -> &wgpu::RenderPipeline {
        &self.render_pipeline
    }
    fn vertex_size(&self) -> usize {
        std::mem::size_of::<Vertex2DTexturedi32>()
    }
}