use std::slice;

use ash::vk;
use glam::{Mat4, Vec3};

use crate::{
    render::{frame::Frame, render_ctx},
    RenderCtx,
};

pub unsafe fn render_frame(ctx: &RenderCtx, frame_index: &mut usize) {
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

    let clear_values = [
        vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [100.0 / 255.0, 149.0 / 255.0, 237.0 / 255.0, 1.0],
            },
        },
        vk::ClearValue {
            depth_stencil: vk::ClearDepthStencilValue {
                depth: 0.0,
                stencil: 0,
            },
        },
    ];

    let render_pass_begin_info = vk::RenderPassBeginInfo::default()
        .render_pass(ctx.render_pass)
        .framebuffer(ctx.framebuffers[image_index as usize])
        .render_area(
            vk::Rect2D::default().extent(
                vk::Extent2D::default()
                    .width(render_ctx::WIDTH)
                    .height(render_ctx::HEIGHT),
            ),
        )
        .clear_values(&clear_values);

    device_loader.cmd_begin_render_pass(
        command_buffer,
        &render_pass_begin_info,
        vk::SubpassContents::INLINE,
    );

    render_frame_inner(ctx, current_frame, image_index as usize);

    device_loader.cmd_end_render_pass(command_buffer);

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

unsafe fn render_frame_inner(ctx: &RenderCtx, current_frame: &Frame, _image_index: usize) {
    let command_buffer = current_frame.command_buffer;

    ctx.device_loader.cmd_bind_pipeline(
        command_buffer,
        vk::PipelineBindPoint::GRAPHICS,
        ctx.pipeline,
    );

    *(current_frame.uniform_buffer.memory.unwrap() as *mut _) =
        Mat4::from_scale(Vec3::new(2.0, 2.0, 1.0));

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
