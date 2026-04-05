use hotham::{
    glam::{self, Vec4Swizzles},
    vk, COLOR_FORMAT,
};

use crate::graphics::{
    allocator::BufferAllocation,
    descriptors::Descriptors,
    pipeline::{Pipeline, PipelineOptions},
};
pub const MAX_LINES: u16 = u16::MAX;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Registers {
    pub view_proj: [glam::Mat4; 2],
    pub viewport_size: glam::Vec2,
}

unsafe impl bytemuck::Pod for Registers {}
unsafe impl bytemuck::Zeroable for Registers {}

pub struct LineRenderer {
    pub lines: BufferAllocation<DebugLine>,
    pub pipeline: Pipeline,
}

impl LineRenderer {
    pub fn new(
        engine: &mut hotham::Engine,
        allocator: &mut super::allocator::Allocator,
        descriptors: &mut super::descriptors::Descriptors,
    ) -> Self {
        let context = &mut engine.vulkan_context;
        let lines = allocator.allocate_buffer(
            MAX_LINES as _,
            vk::BufferUsageFlags::STORAGE_BUFFER,
            context,
        );

        unsafe {
            descriptors.update_storage_buffer_descriptor_set(
                Descriptors::LINES_BINDING,
                lines.handle,
                context,
            )
        };

        let pipeline = Pipeline::new::<Registers>(
            context,
            engine.render_context.render_pass,
            descriptors,
            COLOR_FORMAT,
            include_bytes!("../../shaders/lines.vert.spv"),
            include_bytes!("../../shaders/lines.frag.spv"),
            PipelineOptions {
                depth_write: false,
                ..Default::default()
            },
        );

        Self { lines, pipeline }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct DebugLine {
    start: glam::Vec3,
    end: glam::Vec3,
    colour: glam::Vec3,
}

unsafe impl bytemuck::Pod for DebugLine {}
unsafe impl bytemuck::Zeroable for DebugLine {}

impl DebugLine {
    pub fn new(start: glam::Vec3, end: glam::Vec3, colour: glam::Vec4) -> Self {
        Self {
            start,
            end,
            colour: colour.xyz(),
        }
    }
}
