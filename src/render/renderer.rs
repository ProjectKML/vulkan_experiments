use std::collections::HashSet;
use std::slice;

use ash::vk;
use glam::{Mat4, Vec3};
use winit::event::VirtualKeyCode;

use crate::render::Camera;
use crate::{
    render::{frame::Frame, render_ctx},
    RenderCtx,
};
use crate::render::render_ctx::{HEIGHT, WIDTH};

pub unsafe fn render_frame(ctx: &RenderCtx, frame_index: &mut usize, camera: &Camera) {
    let device_loader = &ctx.device_loader;
    let direct_queue = ctx.direct_queue;
    let swapchain_loader = &ctx.swapchain_loader;
    let swapchain = ctx.swapchain;

    let current_frame = &ctx.frames[*frame_index];

    let present_semaphore = current_frame.present_semaphore;
    let render_semaphore = current_frame.render_semaphore;

    let fence = current_frame.fence;
    device_loader
        .wait_for_fences(slice::from_ref(&fence), true, u64::MAX)
        .unwrap();
    device_loader.reset_fences(slice::from_ref(&fence)).unwrap();

    let command_pool = current_frame.command_pool;
    let command_buffer = current_frame.command_buffer;

    device_loader
        .reset_command_buffer(
            command_buffer,
            vk::CommandBufferResetFlags::RELEASE_RESOURCES,
        )
        .unwrap();
    device_loader
        .reset_command_pool(command_pool, vk::CommandPoolResetFlags::RELEASE_RESOURCES)
        .unwrap();

    let image_index = swapchain_loader
        .acquire_next_image(swapchain, u64::MAX, present_semaphore, vk::Fence::null())
        .unwrap()
        .0;

    let command_buffer_begin_info =
        vk::CommandBufferBeginInfo::default().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

    device_loader
        .begin_command_buffer(command_buffer, &command_buffer_begin_info)
        .unwrap();

    let image = ctx.swapchain_images[image_index as usize];

    let barrier = vk::ImageMemoryBarrier::default()
        .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
        .old_layout(vk::ImageLayout::UNDEFINED)
        .new_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
        .image(image)
        .subresource_range(
            vk::ImageSubresourceRange::default()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .layer_count(1)
                .level_count(1),
        );

    device_loader.cmd_pipeline_barrier(
        command_buffer,
        vk::PipelineStageFlags::TOP_OF_PIPE,
        vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
        vk::DependencyFlags::empty(),
        &[],
        &[],
        slice::from_ref(&barrier),
    );

    let color_attachment = vk::RenderingAttachmentInfo::default()
        .image_view(ctx.swapchain_image_views[image_index as usize])
        .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::STORE)
        .clear_value(vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [100.0 / 255.0, 149.0 / 255.0, 237.0 / 255.0, 1.0],
            },
        });

    let depth_attachment = vk::RenderingAttachmentInfo::default()
        .image_view(ctx.depth_image_view)
        .image_layout(vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::DONT_CARE) //we dont need it atm
        .clear_value(vk::ClearValue {
            depth_stencil: vk::ClearDepthStencilValue {
                depth: 0.0,
                stencil: 0,
            },
        });

    let rendering_info = vk::RenderingInfo::default()
        .render_area(vk::Rect2D::default().extent(vk::Extent2D::default().width(WIDTH).height(HEIGHT)))
        .layer_count(1)
        .color_attachments(slice::from_ref(&color_attachment))
        .depth_attachment(&depth_attachment);

    device_loader.cmd_begin_rendering(command_buffer, &rendering_info);

    render_frame_inner(ctx, current_frame, image_index as usize, camera);

    device_loader.cmd_end_rendering(command_buffer);

    let barrier = vk::ImageMemoryBarrier::default()
        .src_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
        .old_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
        .new_layout(vk::ImageLayout::PRESENT_SRC_KHR)
        .image(image)
        .subresource_range(
            vk::ImageSubresourceRange::default()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .layer_count(1)
                .level_count(1),
        );

    device_loader.cmd_pipeline_barrier(
        command_buffer,
        vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
        vk::PipelineStageFlags::BOTTOM_OF_PIPE,
        vk::DependencyFlags::empty(),
        &[],
        &[],
        slice::from_ref(&barrier),
    );

    device_loader.end_command_buffer(command_buffer).unwrap();

    let wait_semaphores = [present_semaphore];
    let wait_dst_stage_mask = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];

    let submit_info = vk::SubmitInfo::default()
        .wait_semaphores(&wait_semaphores)
        .wait_dst_stage_mask(&wait_dst_stage_mask)
        .command_buffers(slice::from_ref(&command_buffer))
        .signal_semaphores(slice::from_ref(&render_semaphore));

    device_loader
        .queue_submit(direct_queue, slice::from_ref(&submit_info), fence)
        .unwrap();

    let present_info = vk::PresentInfoKHR::default()
        .wait_semaphores(slice::from_ref(&render_semaphore))
        .swapchains(slice::from_ref(&swapchain))
        .image_indices(slice::from_ref(&image_index));

    swapchain_loader
        .queue_present(direct_queue, &present_info)
        .unwrap();
}

unsafe fn render_frame_inner(
    ctx: &RenderCtx,
    current_frame: &Frame,
    _image_index: usize,
    camera: &Camera,
) {
    *(current_frame.uniform_buffer.memory.unwrap() as *mut _) = camera.view_projection_matrix;

    let command_buffer = current_frame.command_buffer;

    ctx.device_loader.cmd_bind_pipeline(
        command_buffer,
        vk::PipelineBindPoint::GRAPHICS,
        ctx.pipeline,
    );

    ctx.device_loader.cmd_bind_descriptor_sets(
        command_buffer,
        vk::PipelineBindPoint::GRAPHICS,
        ctx.pipeline_layout,
        0,
        slice::from_ref(&current_frame.descriptor_set),
        &[],
    );

    let viewport = vk::Viewport::default()
        .width(render_ctx::WIDTH as _)
        .height(render_ctx::HEIGHT as _)
        .max_depth(1.0);
    let scissor = vk::Rect2D::default().extent(vk::Extent2D {
        width: render_ctx::WIDTH,
        height: render_ctx::HEIGHT,
    });

    ctx.device_loader
        .cmd_set_viewport(command_buffer, 0, slice::from_ref(&viewport));
    ctx.device_loader
        .cmd_set_scissor(command_buffer, 0, slice::from_ref(&scissor));

    ctx.mesh_shader_loader
        .cmd_draw_mesh_tasks(command_buffer, 1, 0);
}
