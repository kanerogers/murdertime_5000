mod allocator;
mod components;
mod compute;
mod descriptors;
mod physics;
mod pipeline;
mod renderer;
mod systems;

use crate::{
    components::{DynamicPhysicsBody, KinematicPhysicsBody},
    physics::Physics,
    renderer::{Renderer, SimParams, PARTICLE_COUNT},
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

#[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "on"))]
pub fn main() {
    println!("[FIREFLIES] MAIN!");
    real_main().expect("Error running app!");
    println!("[FIREFLIES] FINISHED! Goodbye!");
}

pub fn real_main() -> HothamResult<()> {
    env_logger::builder()
        .filter_module("murdertime_5000", log::LevelFilter::Trace)
        .init();

    info!("Initialising fireflies..");
    info!("Building engine..");
    let mut engine = Engine::new();
    info!("..done!");

    info!("Building physics..");
    let mut physics = Physics::new();
    info!("..done!");

    // let mut renderer = Renderer::new(&mut engine);
    info!("Initialising app..");
    init(&mut engine)?;
    info!("Done! Entering main loop..");

    // let mut sim_params = SimParams::default();

    while let Ok(tick_data) = engine.update() {
        tick(tick_data, &mut engine, &mut physics);
        engine.finish()?;
    }

    Ok(())
}

fn tick(
    tick_data: TickData,
    engine: &mut Engine,
    physics: &mut Physics,
    // renderer: &mut Renderer,
    // sim_params: &mut SimParams,
) {
    // Gameplay loop
    if tick_data.current_state == xr::SessionState::FOCUSED {
        let mut command_buffer = hecs::CommandBuffer::new();
        // Custom physics
        systems::physics::physics_system(engine, physics, &mut command_buffer);

        // Bzzzt
        hotham::systems::haptics_system(engine);
        hotham::systems::update_global_transform_system(engine);

        command_buffer.run_on(&mut engine.world);
    }

    let views = engine.xr_context.update_views().to_owned();
    // Rendering loop
    unsafe {
        rendering::begin(
            &mut engine.world,
            &mut engine.vulkan_context,
            &mut engine.render_context,
            &views,
            tick_data.swapchain_image_index,
        );

        // PBR Rendering
        rendering::draw_world(&mut engine.vulkan_context, &mut engine.render_context);

        // Fireflies
        {
            // update_sim_params(sim_params, engine, &views);
            // renderer.render(engine, sim_params);
        }

        rendering::end(&mut engine.vulkan_context, &mut engine.render_context);
    }
}

fn update_sim_params(sim_params: &mut SimParams, engine: &mut Engine, views: &[xr::View]) {
    sim_params.left_hand_pos = engine.input_context.left.position();
    sim_params.right_hand_pos = engine.input_context.right.position();
    sim_params.head_pos = engine.input_context.hmd.position();

    // Create transformations to globally oriented stage space
    let global_from_stage = hotham::components::stage::get_global_from_stage(&engine.world);

    // `gos_from_global` is just the inverse of `global_from_stage`'s translation - rotation is ignored.
    let gos_from_global =
        glam::Affine3A::from_translation(glam::Vec3::from(global_from_stage.translation)).inverse();

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

    sim_params.view_proj = view_proj;
    sim_params.camera_right = camera_right;
    sim_params.camera_up = camera_up;
    sim_params.particle_count = PARTICLE_COUNT;
}

fn init(engine: &mut Engine) -> Result<(), hotham::HothamError> {
    let render_context = &mut engine.render_context;
    let vulkan_context = &mut engine.vulkan_context;
    let world = &mut engine.world;

    let glb_buffers: Vec<&[u8]> = vec![
        include_bytes!("../assets/floor.glb"),
        include_bytes!("../assets/damaged_helmet_squished.glb"),
    ];
    let models =
        asset_importer::load_models_from_glb(&glb_buffers, vulkan_context, render_context)?;
    add_floor(&models, world);

    let models =
        asset_importer::load_models_from_glb(&glb_buffers, vulkan_context, render_context)?;
    add_helmet(&models, world);

    // Update global transforms from local transforms before physics_system gets confused
    update_global_transform_system(engine);

    Ok(())
}

fn add_floor(models: &std::collections::HashMap<String, World>, world: &mut World) {
    let entity = asset_importer::add_model_to_world("Floor", models, world, None)
        .expect("Could not find Floor");
    // let collider = Collider::new(SharedShape::halfspace(na::Vector3::y_axis()));
    // let rigid_body = RigidBody {
    //     body_type: BodyType::Fixed,
    //     ..Default::default()
    // };
    world
        .insert_one(entity, KinematicPhysicsBody::new_box(5.0, 0.5, 5.0))
        .unwrap();
}

fn add_helmet(models: &std::collections::HashMap<String, World>, world: &mut World) {
    let helmet = asset_importer::add_model_to_world("Damaged Helmet", models, world, None)
        .expect("Could not find Damaged Helmet");

    {
        let mut local_transform = world.get::<&mut LocalTransform>(helmet).unwrap();
        local_transform.translation.z = -1.;
        local_transform.translation.y = 10.4;
        local_transform.scale = [0.5, 0.5, 0.5].into();
    }

    // let collider = Collider::new(SharedShape::ball(0.35));

    world
        .insert_one(helmet, DynamicPhysicsBody::new_sphere(0.35))
        .unwrap();
}
