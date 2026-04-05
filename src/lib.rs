mod components;
mod graphics;
mod physics;
mod systems;
mod viking;

use std::collections::HashMap;

use crate::{
    components::{DynamicPhysicsBody, KinematicPhysicsBody},
    graphics::renderer::Renderer,
    physics::Physics,
    viking::spawn_vikings,
};
use hotham::{
    asset_importer,
    components::LocalTransform,
    glam,
    hecs::{self, World},
    rendering::camera::Frustum,
    systems::{rendering, update_global_transform_system},
    xr, Engine, HothamResult, TickData,
};
use log::info;

pub const DELTA_TIME: f32 = 1. / 72.;
pub const UNIT_RADIUS: f32 = 0.5;
pub const UNIT_COUNT: usize = 20;
pub const UNIT_HEIGHT: f32 = 1.5;
pub const UNIT_HALF_HEIGHT: f32 = UNIT_HEIGHT / 2.0;
pub const UNIT_MAX_HEALTH: u32 = 8;
pub const SEPARATION_STRENGTH: f32 = 5.0;
pub const UNIT_INITIAL_TARGET_RADIUS: f32 = 20.;
pub const SPAWN_RADIUS: f32 = 10.;

#[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "on"))]
pub fn main() {
    println!("[FIREFLIES] MAIN!");
    real_main().expect("Error running app!");
    println!("[FIREFLIES] FINISHED! Goodbye!");
}

pub fn real_main() -> HothamResult<()> {
    let _ = env_logger::builder()
        .filter_module("murdertime_5000", log::LevelFilter::Trace)
        .try_init();

    info!("Initialising fireflies..");
    info!("Building engine..");
    let mut engine = Engine::new();
    info!("..done!");

    info!("Building physics..");
    let mut physics = Physics::new();
    info!("..done!");

    let mut renderer = Renderer::new(&mut engine);
    info!("Initialising app..");
    init(&mut engine)?;
    info!("Done! Entering main loop..");

    let mut simulation = Simulation::default();

    while let Ok(tick_data) = engine.update() {
        tick(
            tick_data,
            &mut engine,
            &mut physics,
            &mut renderer,
            &mut simulation,
        );
        engine.finish()?;
    }

    Ok(())
}

fn tick(
    tick_data: TickData,
    engine: &mut Engine,
    physics: &mut Physics,
    renderer: &mut Renderer,
    simulation: &mut Simulation,
) {
    // Gameplay loop
    renderer.allocator.transfers_complete();
    let mut debug_lines = Vec::new();

    if tick_data.current_state == xr::SessionState::FOCUSED {
        let mut command_buffer = hecs::CommandBuffer::new();
        // Custom physics
        systems::physics::physics_system(engine, physics, &mut command_buffer, &mut debug_lines);

        systems::unit_movement::unit_movement_system(engine, simulation);

        // Bzzzt
        hotham::systems::haptics_system(engine);
        hotham::systems::update_global_transform_system(engine);
        hotham::systems::skinning_system(engine);

        command_buffer.run_on(&mut engine.world);
    }

    let views = engine.xr_context.update_views().to_owned();
    simulation.update(engine, &views);

    // Rendering loop
    unsafe {
        renderer.update_lines(debug_lines);
        renderer.execute_transfers(engine);
        rendering::begin(
            &mut engine.world,
            &mut engine.vulkan_context,
            &mut engine.render_context,
            &views,
            tick_data.swapchain_image_index,
        );

        // PBR Rendering
        rendering::draw_world(&mut engine.vulkan_context, &mut engine.render_context);

        // Debug lines
        renderer.render(engine, simulation);

        rendering::end(&mut engine.vulkan_context, &mut engine.render_context);
    }
}

#[derive(Debug, Clone, Default)]
pub struct Simulation {
    left_hand_pos: glam::Vec3,
    right_hand_pos: glam::Vec3,
    head_pos: glam::Vec3,
    view_proj: [glam::Mat4; 2],
    camera_up: [glam::Vec3; 2],
    camera_right: [glam::Vec3; 2],
    viewport_size: glam::Vec2,
}

impl Simulation {
    pub fn update(&mut self, engine: &mut Engine, views: &[xr::View]) {
        self.left_hand_pos = engine.input_context.left.position();
        self.right_hand_pos = engine.input_context.right.position();
        self.head_pos = engine.input_context.hmd.position();

        // Create transformations to globally oriented stage space
        let global_from_stage = hotham::components::stage::get_global_from_stage(&engine.world);

        // `gos_from_global` is just the inverse of `global_from_stage`'s translation - rotation is ignored.
        let gos_from_global =
            glam::Affine3A::from_translation(glam::Vec3::from(global_from_stage.translation))
                .inverse();

        let gos_from_stage: glam::Affine3A = gos_from_global * global_from_stage;

        let view_matrices = &engine
            .render_context
            .cameras
            .iter_mut()
            .enumerate()
            .map(|(n, c)| c.update(&views[n], &gos_from_stage))
            .collect::<Vec<_>>();

        let znear = 0.05;
        let view_proj =
            [0, 1].map(|i| Frustum::from(views[i].fov).projection(znear) * view_matrices[i]);
        let camera_rotations = [0, 1].map(|i| {
            engine.render_context.cameras[i]
                .gos_from_view
                .to_scale_rotation_translation()
                .1
        });

        let camera_right = [0, 1].map(|i| camera_rotations[i] * glam::Vec3::X);
        let camera_up = [0, 1].map(|i| camera_rotations[i] * glam::Vec3::Y);

        self.view_proj = view_proj;
        self.camera_right = camera_right;
        self.camera_up = camera_up;

        let resolution = engine.xr_context.swapchain_resolution;
        self.viewport_size = glam::Vec2::new(resolution.width as f32, resolution.height as f32);
    }
}

fn init(engine: &mut Engine) -> Result<HashMap<String, hecs::World>, hotham::HothamError> {
    let render_context = &mut engine.render_context;
    let vulkan_context = &mut engine.vulkan_context;
    let world = &mut engine.world;

    let glb_buffers: Vec<&[u8]> = vec![
        include_bytes!("../assets/floor.glb"),
        include_bytes!("../assets/damaged_helmet_squished.glb"),
        include_bytes!("../assets/1_Viking_Male_1.glb"),
        // include_bytes!("../assets/2_Viking_Male_2.glb"),
        // include_bytes!("../assets/3_Viking_Male_3.glb"),
        // include_bytes!("../assets/4_Viking_Male_4.glb"),
        // include_bytes!("../assets/5_Viking_Male_5.glb"),
        // include_bytes!("../assets/6_Viking_Female_1.glb"),
        // include_bytes!("../assets/7_Viking_Female_2.glb"),
        // include_bytes!("../assets/8_Viking_Female_3.glb"),
        // include_bytes!("../assets/9_Viking_Female_4.glb"),
        // include_bytes!("../assets/10_Viking_Female_5.glb"),
    ];
    let models =
        asset_importer::load_models_from_glb(&glb_buffers, vulkan_context, render_context)?;

    add_floor(&models, world);
    spawn_vikings(engine, &models);

    // Update global transforms from local transforms before physics_system gets confused
    update_global_transform_system(engine);

    Ok(models)
}

fn add_floor(models: &std::collections::HashMap<String, World>, world: &mut World) {
    let entity = asset_importer::add_model_to_world("Floor", models, world, None)
        .expect("Could not find Floor");
    // let collider = Collider::new(SharedShape::halfspace(na::Vector3::y_axis()));
    // let rigid_body = RigidBody {
    //     body_type: BodyType::Fixed,
    //     ..Default::default()
    // };
    //
    {
        let mut transform = world.get::<&mut LocalTransform>(entity).unwrap();
        transform.scale = glam::Vec3::new(2.0, 1.0, 2.0);
    }

    world
        .insert_one(entity, KinematicPhysicsBody::new_box(10.0, 0.5, 10.0))
        .unwrap();
}
