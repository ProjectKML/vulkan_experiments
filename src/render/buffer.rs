use anyhow::Result;
use ash::vk;
use std::sync::Arc;
use vk_mem::{Allocation, AllocationCreateInfo, Allocator, MemoryUsage};

pub struct Buffer {
    pub buffer: vk::Buffer,
    pub size: vk::DeviceSize,
    pub allocation: Allocation,
    pub memory: Option<*mut u8>,

    pub allocator: Arc<Allocator>,
}

impl Buffer {
    pub fn new_cpu_to_gpu(
        allocator: Arc<Allocator>,
        size: vk::DeviceSize,
        usage: vk::BufferUsageFlags,
    ) -> Result<Self> {
        let buffer_create_info = vk::BufferCreateInfo::default().size(size).usage(usage);
        let allocation_create_info = AllocationCreateInfo::new().usage(MemoryUsage::CpuToGpu);
        let buffer =
            unsafe { allocator.create_buffer(&buffer_create_info, &allocation_create_info)? };
        let memory = Some(unsafe { allocator.map_memory(buffer.1)? });
        Ok(Self {
            buffer: buffer.0,
            size,
            allocation: buffer.1,
            memory,
            allocator,
        })
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            if let Some(_) = self.memory {
                self.allocator.unmap_memory(self.allocation);
            }

            self.allocator.destroy_buffer(self.buffer, self.allocation);
        }
    }
}
