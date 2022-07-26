use std::{slice, sync::Arc};

use ash::{vk, Device};

pub struct Frame {
    pub command_pool: vk::CommandPool,
    pub command_buffer: vk::CommandBuffer,

    pub present_semaphore: vk::Semaphore,
    pub render_semaphore: vk::Semaphore,

    pub fence: vk::Fence,

    device: Arc<Device>
}

pub const NUM_FRAMES: usize = 2;

impl Frame {
    pub fn new(device: Arc<Device>) -> Self {
        unsafe {
            let command_pool_create_info = vk::CommandPoolCreateInfo::builder().flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER).queue_family_index(0);

            let command_pool = device.create_command_pool(&command_pool_create_info, None).unwrap();

            let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
                .command_pool(command_pool)
                .level(vk::CommandBufferLevel::PRIMARY)
                .command_buffer_count(1);

            let command_buffer = device.allocate_command_buffers(&command_buffer_allocate_info).unwrap()[0];

            let semaphore_create_info = vk::SemaphoreCreateInfo::builder();

            let present_semaphore = device.create_semaphore(&semaphore_create_info, None).unwrap();
            let render_semaphore = device.create_semaphore(&semaphore_create_info, None).unwrap();

            let fence_create_info = vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);

            let fence = device.create_fence(&fence_create_info, None).unwrap();

            Self {
                command_pool,
                command_buffer,

                present_semaphore,
                render_semaphore,

                fence,

                device
            }
        }
    }
}

impl Drop for Frame {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_fence(self.fence, None);

            self.device.destroy_semaphore(self.render_semaphore, None);
            self.device.destroy_semaphore(self.present_semaphore, None);

            self.device.free_command_buffers(self.command_pool, slice::from_ref(&self.command_buffer));
            self.device.destroy_command_pool(self.command_pool, None);
        }
    }
}
