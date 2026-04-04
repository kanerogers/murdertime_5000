use hotham::glam;
use hotham::vk;
use hotham::Engine;
use rand::Rng;
use rand::RngExt;

use crate::allocator::Allocator;
use crate::allocator::BufferAllocation;
use crate::compute::ComputePipeline;
use crate::descriptors::Descriptors;
use crate::pipeline::Pipeline;
use crate::pipeline::PipelineOptions;

pub const FULL_IMAGE: vk::ImageSubresourceRange = vk::ImageSubresourceRange {
    aspect_mask: vk::ImageAspectFlags::COLOR,
    base_mip_level: 0,
    level_count: vk::REMAINING_MIP_LEVELS,
    base_array_layer: 0,
    layer_count: vk::REMAINING_ARRAY_LAYERS,
};

pub const PARTICLE_COUNT: u32 = 512;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Registers {
    particles: vk::DeviceAddress,
    sim_params: vk::DeviceAddress,
}

unsafe impl bytemuck::Zeroable for Registers {}
unsafe impl bytemuck::Pod for Registers {}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Particle {
    position: glam::Vec3,
    size: f32,
    velocity: glam::Vec3,
    phase: f32,
    colour: glam::Vec3,
    brightness: f32,
}

unsafe impl bytemuck::Zeroable for Particle {}
unsafe impl bytemuck::Pod for Particle {}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SimParams {
    pub view_proj: [glam::Mat4; 2],
    pub camera_right: [glam::Vec3; 2],
    pub camera_up: [glam::Vec3; 2],

    pub right_hand_pos: glam::Vec3,
    pub right_hand_attract: f32,
    pub left_hand_pos: glam::Vec3,
    pub left_hand_repel: f32,
    pub head_pos: glam::Vec3,
    pub head_repel: f32,

    pub dt: f32,
    pub time: f32,
    pub hand_radius: f32,
    pub head_radius: f32,

    pub drag: f32,
    pub wander_strength: f32,
    pub max_speed: f32,
    pub swirl_strength: f32,
    pub particle_count: u32,
}

impl Default for SimParams {
    fn default() -> Self {
        Self {
            view_proj: Default::default(),
            camera_right: Default::default(),
            camera_up: Default::default(),
            right_hand_pos: Default::default(),
            right_hand_attract: 1.8,
            left_hand_pos: Default::default(),
            left_hand_repel: 2.5,
            head_pos: Default::default(),
            head_repel: 2.5,
            dt: Default::default(),
            time: Default::default(),
            hand_radius: 0.35,
            head_radius: 0.35,
            drag: 1.5,
            wander_strength: 0.8,
            max_speed: 0.9,
            swirl_strength: 0.7,
            particle_count: PARTICLE_COUNT,
        }
    }
}

unsafe impl bytemuck::Zeroable for SimParams {}
unsafe impl bytemuck::Pod for SimParams {}

pub struct Renderer {
    compute_pipeline: ComputePipeline<Registers>,
    graphics_pipeline: Pipeline,
    allocator: Allocator,
    sim_param_buffers: [BufferAllocation<SimParams>; 2],
    particle_buffers: [BufferAllocation<Particle>; 2],
}

impl Renderer {
    pub fn new(engine: &mut hotham::Engine) -> Self {
        let context = &engine.vulkan_context;

        let descriptors = Descriptors::new(&engine.vulkan_context);
        let mut allocator = Allocator::new(context);

        // Create sim params buffer
        let sim_param_buffers = [0, 1].map(|_| {
            allocator.allocate_buffer::<SimParams>(1, vk::BufferUsageFlags::UNIFORM_BUFFER, context)
        });

        let head_position = engine.input_context.hmd.position();
        let mut rng = rand::rng();

        // Create particles buffer
        let particle_buffers = [0, 1].map(|_| {
            let mut buffer = allocator.allocate_buffer::<Particle>(
                PARTICLE_COUNT as usize,
                vk::BufferUsageFlags::UNIFORM_BUFFER,
                context,
            );

            for _ in 0..PARTICLE_COUNT {
                buffer.append_one(&spawn_particle(&mut rng, head_position), &mut allocator);
            }

            buffer
        });

        // Create compute pipeline
        log::debug!("Creating compute pipeline..");
        let compute_pipeline =
            ComputePipeline::load(context, include_bytes!("../shaders/fireflies.comp.spv"));
        log::debug!("..done!");

        // Create graphics pipeline
        log::debug!("Creating graphics pipeline..");
        let graphics_pipeline = Pipeline::new::<Registers>(
            context,
            &descriptors,
            hotham::COLOR_FORMAT,
            include_bytes!("../shaders/fireflies.vert.spv"),
            include_bytes!("../shaders/fireflies.frag.spv"),
            PipelineOptions {
                ..Default::default()
            },
        );
        log::debug!("..done!");

        Renderer {
            compute_pipeline,
            graphics_pipeline,
            allocator,
            sim_param_buffers,
            particle_buffers,
        }
    }

    pub fn render(&mut self, engine: &mut Engine, sim_params: &SimParams) {
        let frame_index = engine.render_context.frame_index;
        let command_buffer = engine.render_context.frames[frame_index].command_buffer;
        let context = &engine.vulkan_context;
        let allocator = &mut self.allocator;
        let device = &context.device;

        // - Get per-frame buffers
        let sim_params_buffer = &mut self.sim_param_buffers[frame_index];
        let particle_buffer = &self.particle_buffers[frame_index];

        // - Update sim params
        sim_params_buffer.clear();
        sim_params_buffer.append_one(sim_params, allocator);

        allocator.execute_transfers(command_buffer, context);

        // - Bind compute pipeline
        unsafe {
            context.device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::COMPUTE,
                self.compute_pipeline.handle,
            );
        }

        // - Update push constants
        self.compute_pipeline.update_registers(
            &Registers {
                particles: particle_buffer.device_address,
                sim_params: sim_params_buffer.device_address,
            },
            context,
            command_buffer,
        );

        // - Run compute shader
        unsafe {
            let group_count_x = (PARTICLE_COUNT + 63) / 64;
            device.cmd_dispatch(command_buffer, group_count_x, 1, 1);
        }

        // - Barrier compute / graphics
        unsafe {
            device.cmd_pipeline_barrier2(
                command_buffer,
                &vk::DependencyInfo::default().buffer_memory_barriers(&[
                    vk::BufferMemoryBarrier2::default()
                        .buffer(particle_buffer.handle)
                        .src_stage_mask(vk::PipelineStageFlags2::COMPUTE_SHADER)
                        .src_access_mask(vk::AccessFlags2::SHADER_WRITE)
                        .dst_stage_mask(vk::PipelineStageFlags2::VERTEX_SHADER)
                        .dst_access_mask(vk::AccessFlags2::SHADER_READ)
                        .size(vk::WHOLE_SIZE),
                ]),
            );
        }
        // - Bind graphics pipeline
        unsafe {
            device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.graphics_pipeline.handle,
            );
        }
        // - Draw quads
        unsafe {
            device.cmd_draw(command_buffer, PARTICLE_COUNT * 6, 1, 0, 0);
        }
    }
}

fn random_unit_vector(rng: &mut impl Rng) -> glam::Vec3 {
    loop {
        let v = glam::Vec3::new(
            rng.random_range(-1.0..1.0),
            rng.random_range(-1.0..1.0),
            rng.random_range(-1.0..1.0),
        );
        let len2 = v.length_squared();
        if len2 > 0.0001 && len2 <= 1.0 {
            return v / len2.sqrt();
        }
    }
}

fn spawn_particle(rng: &mut impl Rng, head_pos: glam::Vec3) -> Particle {
    let center = head_pos + glam::Vec3::new(0.0, -0.3, 0.0);

    let dir = random_unit_vector(rng);
    let radius = rng.random_range(0.35..1.2);
    let position = center + dir * radius;

    let velocity = random_unit_vector(rng) * rng.random_range(0.02..0.08);

    let size = rng.random_range(0.015..0.03);
    let phase = rng.random_range(0.0..std::f32::consts::TAU);

    let colour = glam::Vec3::new(
        rng.random_range(0.7..1.0),
        rng.random_range(0.8..1.0),
        rng.random_range(0.4..0.8),
    );

    let brightness = rng.random_range(0.4..0.8);

    Particle {
        position,
        size,
        velocity,
        phase,
        colour,
        brightness,
    }
}
