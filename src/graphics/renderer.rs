use hotham::glam;
use hotham::vk;
use hotham::Engine;

use crate::graphics::line_renderer;
use crate::graphics::line_renderer::DebugLine;
use crate::graphics::line_renderer::LineRenderer;
use crate::Simulation;

use super::{allocator::Allocator, descriptors::Descriptors};

pub const FULL_IMAGE: vk::ImageSubresourceRange = vk::ImageSubresourceRange {
    aspect_mask: vk::ImageAspectFlags::COLOR,
    base_mip_level: 0,
    level_count: vk::REMAINING_MIP_LEVELS,
    base_array_layer: 0,
    layer_count: vk::REMAINING_ARRAY_LAYERS,
};

pub const PARTICLE_COUNT: u32 = 512;

pub struct Renderer {
    descriptors: Descriptors,
    pub allocator: Allocator,
    pub line_renderer: LineRenderer,
}

impl Renderer {
    pub fn new(engine: &mut hotham::Engine) -> Self {
        let context = &engine.vulkan_context;

        let mut descriptors = Descriptors::new(&engine.vulkan_context);
        let mut allocator = Allocator::new(context);

        log::debug!("Creating line renderer..");
        let line_renderer = LineRenderer::new(engine, &mut allocator, &mut descriptors);
        log::debug!("..done!");

        Renderer {
            allocator,
            descriptors,
            line_renderer,
        }
    }

    pub fn update_lines(
        &mut self,
        mut debug_lines: Vec<DebugLine>,
        global_from_stage: glam::Affine3A,
    ) {
        self.line_renderer.lines.clear();
        let gos_from_global =
            glam::Affine3A::from_translation(glam::Vec3::from(global_from_stage.translation))
                .inverse();

        for mut line in debug_lines.drain(..) {
            line.start = gos_from_global.transform_point3(line.start);
            line.end = gos_from_global.transform_point3(line.end);

            self.line_renderer
                .lines
                .append_one(&line, &mut self.allocator);
        }
    }

    pub fn execute_transfers(&mut self, engine: &mut Engine) {
        let frame_index = engine.render_context.frame_index;
        let command_buffer = engine.render_context.frames[frame_index].command_buffer;
        let context = &engine.vulkan_context;
        let allocator = &mut self.allocator;

        allocator.execute_transfers(command_buffer, context);
    }

    pub fn render(&mut self, engine: &mut Engine, simulation: &Simulation) {
        self.draw_lines(engine, simulation);
    }

    pub fn draw_lines(&self, engine: &mut Engine, simulation: &Simulation) {
        let frame_index = engine.render_context.frame_index;
        let frame = &engine.render_context.frames[frame_index];
        let command_buffer = frame.command_buffer;
        let context = &engine.vulkan_context;
        let device = &context.device;
        let line_renderer = &self.line_renderer;

        unsafe {
            device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                line_renderer.pipeline.handle,
            );

            let render_area = engine.xr_context.swapchain_resolution;

            // Set the dynamic state
            device.cmd_set_scissor(command_buffer, 0, &[render_area.into()]);
            device.cmd_set_viewport(
                command_buffer,
                0,
                &[vk::Viewport::default()
                    .width(render_area.width as _)
                    .height(render_area.height as _)
                    .max_depth(1.)],
            );

            line_renderer.pipeline.update_registers(
                &line_renderer::Registers {
                    view_proj: simulation.view_proj,
                    viewport_size: simulation.viewport_size,
                },
                context,
                command_buffer,
            );

            line_renderer
                .pipeline
                .bind_descriptor_sets(context, command_buffer);

            device.cmd_draw(
                command_buffer,
                line_renderer.lines.len() as u32 * 6,
                1,
                0,
                0,
            );
        }
    }
}
