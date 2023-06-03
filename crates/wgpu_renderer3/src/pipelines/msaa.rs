/*
pub fn msaa(renderer: &mut crate::Renderer) {
    let texture_temporary;
    unsafe {
        if texture_temporary_static.is_none() {
            texture_temporary_static = Some(renderer.device.create_texture(&wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d { width : renderer.swap_chain_size.width, height: renderer.swap_chain_size.height, depth: 1 },
                array_layer_count: 1,
                mip_level_count: 1,
                sample_count: 2,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Bgra8Unorm,
                usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            }));
        }
        texture_temporary = texture_temporary_static.as_ref().unwrap();
    }

    let temporary_attachment_view = texture_temporary.create_default_view();




    let image_view = &renderer.actual_frame.frame.as_ref().unwrap().view;
    let mut command_encoder = renderer.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    {

        let depth_buffer_texture_view = renderer.depth_buffer_texture.create_default_view();
        let depth_buffer = wgpu::RenderPassDepthStencilAttachmentDescriptor {
            attachment: &depth_buffer_texture_view,
            depth_load_op: wgpu::LoadOp::Clear,
            depth_store_op: wgpu::StoreOp::Store,
            clear_depth: 0.0,
            stencil_load_op: wgpu::LoadOp::Clear,
            stencil_store_op: wgpu::StoreOp::Store,
            clear_stencil: 0,
        };
        
        let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: &temporary_attachment_view,
                resolve_target: Some(image_view),
                load_op: wgpu::LoadOp::Clear,
                store_op: wgpu::StoreOp::Store,
                clear_color: wgpu::Color {
                    r: 0.1,
                    g: 0.1,
                    b: 0.1,
                    a: 1.0,
                },
            }],
            depth_stencil_attachment: Some(depth_buffer),
        });
        render_pass.set_bind_group(0, &renderer.bind_group, &[]);
        render_pass.set_pipeline(&renderer.render_pipeline);
        render_pass.set_vertex_buffer(0, &renderer.vertex_buffer, 0, renderer.actual_frame.vertex_size as u64);
        render_pass.draw(0..renderer.actual_frame.vertex_count as u32, 0..1);
    }
    renderer.queue.submit(&[command_encoder.finish()]);
}
*/