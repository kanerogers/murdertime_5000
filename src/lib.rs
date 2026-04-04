mod allocator;
mod compute;
mod descriptors;
mod pipeline;
mod renderer;

use hotham::{
    glam, rendering::camera::Frustum, systems::rendering, xr, Engine, HothamResult, TickData,
};
use log::info;

use crate::renderer::{Renderer, SimParams, PARTICLE_COUNT};

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

    info!("Initialising app..");
    let mut renderer = Renderer::new(&mut engine);
    init(&mut engine)?;
    info!("Done! Entering main loop..");

    let mut sim_params = SimParams::default();

    while let Ok(tick_data) = engine.update() {
        tick(tick_data, &mut engine, &mut renderer, &mut sim_params);
        engine.finish()?;
    }

    Ok(())
}

fn tick(
    tick_data: TickData,
    engine: &mut Engine,
    renderer: &mut Renderer,
    sim_params: &mut SimParams,
) {
    if tick_data.current_state == xr::SessionState::FOCUSED {}

    let views = engine.xr_context.update_views().to_owned();

    unsafe {
        rendering::begin(
            &mut engine.world,
            &mut engine.vulkan_context,
            &mut engine.render_context,
            &views,
            tick_data.swapchain_image_index,
        );
        update_sim_params(sim_params, engine, &views);
        renderer.render(engine, sim_params);
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

fn init(_engine: &mut Engine) -> Result<(), hotham::HothamError> {
    Ok(())
}
