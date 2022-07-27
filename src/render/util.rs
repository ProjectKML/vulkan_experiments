use std::{ffi::CStr, fs::File, io::Read, path::Path, slice};

use crate::render::render_ctx::{DEPTH_FORMAT, SWAPCHAIN_FORMAT};
use anyhow::Result;
use ash::{vk, Device};
use vk_mem::{Allocation, AllocationCreateInfo, Allocator, MemoryUsage};

pub fn create_depth_image(
    device: &Device,
    allocator: &Allocator,
    width: u32,
    height: u32,
    format: vk::Format,
) -> Result<(vk::Image, Allocation, vk::ImageView)> {
    let image_create_info = vk::ImageCreateInfo::default()
        .image_type(vk::ImageType::TYPE_2D)
        .format(format)
        .extent(vk::Extent3D::default().width(width).height(height).depth(1))
        .mip_levels(1)
        .array_layers(1)
        .samples(vk::SampleCountFlags::TYPE_1)
        .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
        .initial_layout(vk::ImageLayout::UNDEFINED);

    let allocation_create_info = AllocationCreateInfo::new().usage(MemoryUsage::GpuOnly);
    let image = unsafe { allocator.create_image(&image_create_info, &allocation_create_info)? };

    let image_view_create_info = vk::ImageViewCreateInfo::default()
        .image(image.0)
        .view_type(vk::ImageViewType::TYPE_2D)
        .format(format)
        .components(Default::default())
        .subresource_range(
            vk::ImageSubresourceRange::default()
                .aspect_mask(vk::ImageAspectFlags::DEPTH)
                .level_count(1)
                .layer_count(1),
        );

    let image_view = unsafe { device.create_image_view(&image_view_create_info, None) }?;

    Ok((image.0, image.1, image_view))
}

pub fn create_shader_module(device: &Device, path: impl AsRef<Path>) -> Result<vk::ShaderModule> {
    let mut file = File::open(path)?;

    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    unsafe {
        let shader_module_create_info = vk::ShaderModuleCreateInfo::default().code(
            slice::from_raw_parts(buffer.as_ptr().cast(), buffer.len() >> 2),
        );

        Ok(device.create_shader_module(&shader_module_create_info, None)?)
    }
}

pub unsafe fn create_mesh_pipeline(
    device: &Device,
    mesh_shader: vk::ShaderModule,
    task_shader: Option<vk::ShaderModule>,
    fragment_shader: vk::ShaderModule,
    layout: vk::PipelineLayout,
) -> Result<vk::Pipeline> {
    let mut pipeline_rendering_create_info = vk::PipelineRenderingCreateInfo::default()
        .color_attachment_formats(&[SWAPCHAIN_FORMAT])
        .depth_attachment_format(DEPTH_FORMAT);

    let mut shader_stage_create_infos = vec![
        vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::MESH_NV)
            .module(mesh_shader)
            .name(CStr::from_bytes_with_nul_unchecked(b"main\0")),
        vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(fragment_shader)
            .name(CStr::from_bytes_with_nul_unchecked(b"main\0")),
    ];
    if let Some(task_shader) = task_shader {
        shader_stage_create_infos.push(
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::TASK_NV)
                .module(task_shader)
                .name(CStr::from_bytes_with_nul_unchecked(b"main\0")),
        )
    }

    let input_assembly_state_create_info = vk::PipelineInputAssemblyStateCreateInfo::default()
        .topology(vk::PrimitiveTopology::TRIANGLE_LIST);

    let viewport = vk::Viewport::default()
        .width(1.0)
        .height(1.0)
        .max_depth(1.0);

    let scissor = vk::Rect2D::default().extent(vk::Extent2D {
        width: 1,
        height: 1,
    });

    let viewport_state_create_info = vk::PipelineViewportStateCreateInfo::default()
        .viewports(slice::from_ref(&viewport))
        .scissors(slice::from_ref(&scissor));

    let rasterization_state_create_info =
        vk::PipelineRasterizationStateCreateInfo::default().line_width(1.0);

    let depth_stencil_state_create_info = vk::PipelineDepthStencilStateCreateInfo::default()
        .depth_test_enable(true)
        .depth_write_enable(true)
        .depth_compare_op(vk::CompareOp::GREATER_OR_EQUAL);

    let multisample_state_create_info = vk::PipelineMultisampleStateCreateInfo::default()
        .rasterization_samples(vk::SampleCountFlags::TYPE_1);

    let blend_attachment_state = vk::PipelineColorBlendAttachmentState::default()
        .color_write_mask(vk::ColorComponentFlags::RGBA);

    let color_blend_state_create_info = vk::PipelineColorBlendStateCreateInfo::default()
        .attachments(slice::from_ref(&blend_attachment_state));

    let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
    let dynamic_state_create_info =
        vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);

    let graphics_pipeline_create_info = vk::GraphicsPipelineCreateInfo::default()
        .stages(&shader_stage_create_infos)
        .input_assembly_state(&input_assembly_state_create_info)
        .viewport_state(&viewport_state_create_info)
        .rasterization_state(&rasterization_state_create_info)
        .depth_stencil_state(&depth_stencil_state_create_info)
        .multisample_state(&multisample_state_create_info)
        .color_blend_state(&color_blend_state_create_info)
        .dynamic_state(&dynamic_state_create_info)
        .layout(layout)
        .push_next(&mut pipeline_rendering_create_info);

    Ok(device
        .create_graphics_pipelines(
            vk::PipelineCache::null(),
            slice::from_ref(&graphics_pipeline_create_info),
            None,
        )
        .unwrap()[0])
}
