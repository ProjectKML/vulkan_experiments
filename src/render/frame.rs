use std::{mem, slice, sync::Arc};

use crate::render::Buffer;
use ash::vk::{DescriptorBufferInfo, DescriptorPoolSize};
use ash::{vk, Device};
use glam::Mat4;
use vk_mem::Allocator;

pub struct Frame {
    pub command_pool: vk::CommandPool,
    pub command_buffer: vk::CommandBuffer,

    pub present_semaphore: vk::Semaphore,
    pub render_semaphore: vk::Semaphore,

    pub fence: vk::Fence,
    pub uniform_buffer: Buffer,

    pub descriptor_pool: vk::DescriptorPool,
    pub descriptor_set: vk::DescriptorSet,

    device: Arc<Device>,
}

pub const NUM_FRAMES: usize = 2;

impl Frame {
    pub fn new(
        device: Arc<Device>,
        allocator: Arc<Allocator>,
        descriptor_set_layout: vk::DescriptorSetLayout,
    ) -> Self {
        unsafe {
            let command_pool_create_info = vk::CommandPoolCreateInfo::default()
                .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
                .queue_family_index(0);

            let command_pool = device
                .create_command_pool(&command_pool_create_info, None)
                .unwrap();

            let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::default()
                .command_pool(command_pool)
                .level(vk::CommandBufferLevel::PRIMARY)
                .command_buffer_count(1);

            let command_buffer = device
                .allocate_command_buffers(&command_buffer_allocate_info)
                .unwrap()[0];

            let semaphore_create_info = vk::SemaphoreCreateInfo::default();

            let present_semaphore = device
                .create_semaphore(&semaphore_create_info, None)
                .unwrap();
            let render_semaphore = device
                .create_semaphore(&semaphore_create_info, None)
                .unwrap();

            let fence_create_info =
                vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);

            let fence = device.create_fence(&fence_create_info, None).unwrap();

            let uniform_buffer = Buffer::new_cpu_to_gpu(
                allocator.clone(),
                mem::size_of::<Mat4>() as vk::DeviceSize,
                vk::BufferUsageFlags::UNIFORM_BUFFER,
            )
            .unwrap();

            let pool_size = DescriptorPoolSize::default()
                .descriptor_count(1)
                .ty(vk::DescriptorType::UNIFORM_BUFFER);

            let descriptor_pool_create_info = vk::DescriptorPoolCreateInfo::default()
                .max_sets(1)
                .pool_sizes(slice::from_ref(&pool_size));

            let descriptor_pool = device
                .create_descriptor_pool(&descriptor_pool_create_info, None)
                .unwrap();

            let descriptor_set_allocate_info = vk::DescriptorSetAllocateInfo::default()
                .descriptor_pool(descriptor_pool)
                .set_layouts(slice::from_ref(&descriptor_set_layout));

            let descriptor_set = device
                .allocate_descriptor_sets(&descriptor_set_allocate_info)
                .unwrap()[0];

            let descriptor_buffer_info = DescriptorBufferInfo::default()
                .buffer(uniform_buffer.buffer)
                .range(uniform_buffer.size);

            let write_descriptor_set = vk::WriteDescriptorSet::default()
                .dst_set(descriptor_set)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(slice::from_ref(&descriptor_buffer_info));

            device.update_descriptor_sets(slice::from_ref(&write_descriptor_set), &[]);

            Self {
                command_pool,
                command_buffer,

                present_semaphore,
                render_semaphore,

                fence,
                uniform_buffer,
                descriptor_pool,
                descriptor_set,

                device,
            }
        }
    }
}

impl Drop for Frame {
    fn drop(&mut self) {
        unsafe {
            self.device
                .destroy_descriptor_pool(self.descriptor_pool, None);

            self.device.destroy_fence(self.fence, None);

            self.device.destroy_semaphore(self.render_semaphore, None);
            self.device.destroy_semaphore(self.present_semaphore, None);

            self.device
                .free_command_buffers(self.command_pool, slice::from_ref(&self.command_buffer));
            self.device.destroy_command_pool(self.command_pool, None);
        }
    }
}
